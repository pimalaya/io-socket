use std::io::{BufReader, Read};

use io_socket::{
    coroutines::read_to_end::{ReadSocketToEnd, ReadSocketToEndResult},
    io::{SocketInput, SocketOutput},
};

#[test]
fn read_to_end() {
    let _ = env_logger::try_init();

    let mut reader = BufReader::new("abcdef".as_bytes());

    // 4-byte inner buffer, 6-byte source — needs two reads + EOF.
    let mut read = ReadSocketToEnd::with_capacity(4);
    let mut arg = None;

    let buf = loop {
        match read.resume(arg.take()) {
            ReadSocketToEndResult::Ok { buf } => break buf,
            ReadSocketToEndResult::Io {
                input: SocketInput::Read { mut buf },
            } => {
                let n = reader.read(&mut buf).unwrap();
                arg = Some(SocketOutput::Read { buf, n });
            }
            other => unreachable!("Unexpected result: {other:?}"),
        }
    };

    assert_eq!(buf, b"abcdef");
}
