#![cfg(feature = "std-stream")]

use std::{
    env,
    io::{Read, Write, stdin, stdout},
    net::TcpStream,
    sync::Arc,
};

use io_socket::{
    coroutines::{read::*, write::*},
    runtimes::std_stream::handle,
};
use memchr::memmem;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::ConfigVerifierExt;
use url::Url;

fn main() {
    env_logger::init();

    let url: Url = match env::var("URL") {
        Ok(url) => url.parse().unwrap(),
        Err(_) => read_line("URL?").parse().unwrap(),
    };

    let mut stream = connect(&url);

    let request = format!(
        "GET {} HTTP/1.0\r\nHost: {}:{}\r\n\r\n",
        url.path(),
        url.host_str().unwrap(),
        url.port_or_known_default().unwrap(),
    );

    println!("----------------");
    println!("{}", request.trim());
    println!("----------------");

    let mut arg = None;
    let mut write = SocketWrite::new(request.into_bytes());

    loop {
        match write.resume(arg.take()) {
            SocketWriteResult::Ok { .. } => break,
            SocketWriteResult::Io { input } => arg = Some(handle(&mut stream, input).unwrap()),
            SocketWriteResult::Err { err } => panic!("{err}"),
            SocketWriteResult::Eof => panic!("reached unexpected EOF"),
        }
    }

    let mut response = Vec::new();

    loop {
        let mut arg = None;
        let mut read = SocketRead::new();

        let (buf, n) = loop {
            match read.resume(arg.take()) {
                SocketReadResult::Ok { buf, n } => break (buf, n),
                SocketReadResult::Io { input } => arg = Some(handle(&mut stream, input).unwrap()),
                SocketReadResult::Eof => panic!("reached unexpected EOF"),
                SocketReadResult::Err { err } => panic!("{err}"),
            }
        };

        let bytes = &buf[..n];

        match memmem::find(bytes, &[b'\r', b'\n', b'\r', b'\n']) {
            None => {
                response.extend(bytes);
                continue;
            }
            Some(n) => {
                response.extend(&bytes[..n]);
                break;
            }
        }
    }

    println!("{}", String::from_utf8_lossy(&response));
    println!("----------------");
}

fn read_line(prompt: &str) -> String {
    print!("{prompt} ");
    stdout().flush().unwrap();

    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();

    line.trim().to_owned()
}

trait Stream: Read + Write {}
impl<T: Read + Write> Stream for T {}

fn connect(url: &Url) -> Box<dyn Stream> {
    let domain = url.domain().unwrap();

    if url.scheme().eq_ignore_ascii_case("https") {
        let config = ClientConfig::with_platform_verifier().unwrap();
        let server_name = domain.to_string().try_into().unwrap();
        let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
        let tcp = TcpStream::connect((domain.to_string(), 443)).unwrap();
        let tls = StreamOwned::new(conn, tcp);
        Box::new(tls)
    } else {
        let tcp = TcpStream::connect((domain.to_string(), 80)).unwrap();
        Box::new(tcp)
    }
}
