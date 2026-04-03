#![cfg(feature = "std-udp-socket")]

use std::{
    env,
    net::{Ipv4Addr, UdpSocket},
};

use io_socket::{
    coroutines::{read::*, write::*},
    runtimes::std_udp_socket::handle,
};

/// Google's public DNS resolver.
const DNS_SERVER: &str = "8.8.8.8:53";

fn main() {
    env_logger::init();

    let domain = env::var("DOMAIN").unwrap_or_else(|_| "pimalaya.org".to_owned());

    println!("Querying A records for {domain:?} via {DNS_SERVER} …");

    // Build and send the DNS query over a connected UDP socket.
    // DNS over UDP: one datagram out, one datagram back (RFC 1035 §4.2.1).
    let query = encode_a_query(0x1337, &domain);

    let mut socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    // Connecting a UDP socket sets the default peer; send/recv no longer
    // need an explicit address.
    socket.connect(DNS_SERVER).unwrap();

    // Send the query datagram.
    let mut arg = None;
    let mut write = WriteSocket::new(query);

    loop {
        match write.resume(arg.take()) {
            WriteSocketResult::Ok { .. } => break,
            WriteSocketResult::Io { input } => arg = Some(handle(&mut socket, input).unwrap()),
            WriteSocketResult::Err { err } => panic!("write error: {err}"),
            WriteSocketResult::Eof => panic!("reached unexpected EOF"),
        }
    }

    // Receive the response datagram.
    // A single recv() returns the entire DNS response (datagrams are atomic).
    let mut arg = None;
    let mut read = ReadSocket::new();

    let (buf, n) = loop {
        match read.resume(arg.take()) {
            ReadSocketResult::Ok { buf, n } => break (buf, n),
            ReadSocketResult::Io { input } => arg = Some(handle(&mut socket, input).unwrap()),
            ReadSocketResult::Err { err } => panic!("read error: {err}"),
            ReadSocketResult::Eof => panic!("reached unexpected EOF"),
        }
    };

    let addrs = decode_a_records(&buf[..n]);

    if addrs.is_empty() {
        println!("No A records found.");
    } else {
        for addr in addrs {
            println!("  A  {addr}");
        }
    }
}

/// Encodes a DNS query message for A records of `domain` (RFC 1035 §4).
fn encode_a_query(id: u16, domain: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // Header (12 bytes, RFC 1035 §4.1.1).
    buf.extend_from_slice(&id.to_be_bytes()); // ID: echoed back in the response
    buf.extend_from_slice(&[0x01, 0x00]); // Flags: QR=0 (query), RD=1 (recursion desired)
    buf.extend_from_slice(&[0x00, 0x01]); // QDCOUNT: 1 question
    buf.extend_from_slice(&[0x00, 0x00]); // ANCOUNT: 0 (query has no answers)
    buf.extend_from_slice(&[0x00, 0x00]); // NSCOUNT: 0
    buf.extend_from_slice(&[0x00, 0x00]); // ARCOUNT: 0

    // Question section: QNAME encoded as length-prefixed labels (RFC 1035 §3.1).
    // "pimalaya.org" → \x08pimalaya\x03org\x00
    for label in domain.split('.') {
        buf.push(label.len() as u8); // label length
        buf.extend_from_slice(label.as_bytes()); // label bytes
    }

    buf.push(0x00); // root label terminates the name

    buf.extend_from_slice(&[0x00, 0x01]); // QTYPE:  1 = A (host address)
    buf.extend_from_slice(&[0x00, 0x01]); // QCLASS: 1 = IN (internet)

    buf
}

/// Parses IPv4 addresses from the answer section of a DNS response.
fn decode_a_records(buf: &[u8]) -> Vec<Ipv4Addr> {
    if buf.len() < 12 {
        return vec![];
    }

    // ANCOUNT is at bytes 6–7 of the header.
    let ancount = u16::from_be_bytes([buf[6], buf[7]]) as usize;

    // Skip the header (12 bytes), then skip the question section.
    let mut pos = 12;
    pos = skip_name(buf, pos); // QNAME
    pos += 4; // QTYPE + QCLASS

    let mut addrs = Vec::new();

    for _ in 0..ancount {
        pos = skip_name(buf, pos); // RR NAME (usually a compression pointer)

        if pos + 10 > buf.len() {
            break;
        }

        let rtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        pos += 2; // TYPE
        pos += 2; // CLASS
        pos += 4; // TTL
        let rdlength = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
        pos += 2; // RDLENGTH

        // TYPE 1 = A record: RDATA is a 4-byte IPv4 address.
        if rtype == 1 && rdlength == 4 && pos + 4 <= buf.len() {
            addrs.push(Ipv4Addr::new(
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ));
        }

        pos += rdlength;
    }

    addrs
}

/// Advances `pos` past a DNS name field.
///
/// Handles both label sequences (`\x03www\x07example\x03com\x00`) and
/// compression pointers (`0xC0 XX`), as defined in RFC 1035 §4.1.4.
fn skip_name(buf: &[u8], mut pos: usize) -> usize {
    loop {
        if pos >= buf.len() {
            break;
        }
        // High two bits set → 2-byte compression pointer; name ends here.
        if buf[pos] & 0xC0 == 0xC0 {
            pos += 2;
            break;
        }
        let len = buf[pos] as usize;
        pos += 1;
        if len == 0 {
            break; // root label: end of name
        }
        pos += len;
    }
    pos
}
