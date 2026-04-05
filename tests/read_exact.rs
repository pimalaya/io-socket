use std::io::{BufReader, Read};

use io_socket::{
    coroutines::read_exact::{SocketReadExact, SocketReadExactError, SocketReadExactResult},
    io::{SocketInput, SocketOutput},
};

#[test]
fn read_exact_smaller_capacity() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    // Read 4 bytes with a 3-byte inner buffer — needs two iterations.
    let mut read = SocketReadExact::with_capacity(3, 4);
    let mut arg = None;

    let buf = loop {
        match read.resume(arg.take()) {
            SocketReadExactResult::Ok { buf } => break buf,
            SocketReadExactResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => unreachable!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(&buf, b"abcd");

    // Ensure only 4 bytes were consumed from the reader.
    let mut remaining = vec![0; 4];
    let n = reader.read(&mut remaining).unwrap();
    assert_eq!(n, 2);
    assert_eq!(&remaining[..n], b"ef");
}

#[test]
fn read_exact_bigger_capacity() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    // Read 4 bytes with a 5-byte inner buffer — buffer is trimmed to 4.
    let mut read = SocketReadExact::with_capacity(5, 4);
    let mut arg = None;

    let buf = loop {
        match read.resume(arg.take()) {
            SocketReadExactResult::Ok { buf } => break buf,
            SocketReadExactResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => unreachable!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(&buf, b"abcd");

    // Ensure only 4 bytes were consumed from the reader.
    let mut remaining = vec![0; 4];
    let n = reader.read(&mut remaining).unwrap();
    assert_eq!(n, 2);
    assert_eq!(&remaining[..n], b"ef");
}

#[test]
fn read_exact_0() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    // max = 0 with pre-seeded bytes: should complete immediately without any I/O.
    let mut read = SocketReadExact::with_capacity(5, 0);
    read.extend("123".as_bytes().iter().copied());

    let mut arg = None;

    let buf = loop {
        match read.resume(arg.take()) {
            SocketReadExactResult::Ok { buf } => break buf,
            SocketReadExactResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => unreachable!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(buf, b"123");
}

#[test]
fn read_eof() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    // Request 8 bytes from a 6-byte source — must error with UnexpectedEof.
    let mut read = SocketReadExact::new(8);
    let mut arg = None;

    loop {
        match read.resume(arg.take()) {
            SocketReadExactResult::Err {
                err: SocketReadExactError::UnexpectedEof(2, 8, buf),
            } => {
                break assert_eq!(buf, b"abcdef");
            }
            SocketReadExactResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => unreachable!("Unexpected result: {other:?}"),
        }
    }
}
