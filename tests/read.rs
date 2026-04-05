use std::io::{BufReader, Read};

use io_socket::{
    coroutines::read::{SocketRead, SocketReadResult},
    io::{SocketInput, SocketOutput},
};

#[test]
fn read() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    let mut read = SocketRead::with_capacity(4);
    let mut arg = None;

    let (buf, n) = loop {
        match read.resume(arg.take()) {
            SocketReadResult::Ok { buf, n } => break (buf, n),
            SocketReadResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => panic!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(&buf[..n], b"abcd");
    read.replace(buf);

    let (buf, n) = loop {
        match read.resume(arg.take()) {
            SocketReadResult::Ok { buf, n } => break (buf, n),
            SocketReadResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => panic!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(&buf[..n], b"ef");
    read.replace(buf);

    loop {
        match read.resume(arg.take()) {
            SocketReadResult::Eof => break,
            SocketReadResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => panic!("Unexpected result: {other:?}"),
        }
    }
}
