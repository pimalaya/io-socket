#![cfg(feature = "tokio-stream")]

use std::env;

use io_socket::{
    coroutines::{
        read::{SocketRead, SocketReadResult},
        write::{SocketWrite, SocketWriteResult},
    },
    runtimes::tokio_stream::handle,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout, stdin, stdout},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut stdout = stdout();

    let host = match env::var("HOST") {
        Ok(host) => host,
        Err(_) => prompt(&mut stdout, "TCP server host?").await,
    };

    let port: u16 = match env::var("PORT") {
        Ok(port) => port.parse().unwrap(),
        Err(_) => prompt(&mut stdout, "TCP server port?")
            .await
            .parse()
            .unwrap(),
    };

    let mut tcp = TcpStream::connect((host.as_str(), port)).await.unwrap();

    stdout.write_all(b"\nReceived greeting:\n").await.unwrap();

    let mut arg = None;
    let mut read = SocketRead::new();

    let (buf, n) = loop {
        match read.resume(arg.take()) {
            SocketReadResult::Ok { buf, n } => break (buf, n),
            SocketReadResult::Io { input } => arg = Some(handle(&mut tcp, input).await.unwrap()),
            SocketReadResult::Err { err } => panic!("{err}"),
            SocketReadResult::Eof => panic!("reached unexpected EOF"),
        }
    };

    let mut lines = BufReader::new(&buf[..n]).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        stdout
            .write_all(format!("S: {line}\n").as_bytes())
            .await
            .unwrap();
    }

    loop {
        stdout.write_all(b"\n").await.unwrap();

        let mut data = prompt(&mut stdout, "C:").await;
        data.push_str("\r\n");

        let mut arg = None;
        let mut write = SocketWrite::new(data.into_bytes());

        loop {
            match write.resume(arg.take()) {
                SocketWriteResult::Ok { .. } => break,
                SocketWriteResult::Io { input } => {
                    arg = Some(handle(&mut tcp, input).await.unwrap())
                }
                SocketWriteResult::Err { err } => panic!("{err}"),
                SocketWriteResult::Eof => panic!("reached unexpected EOF"),
            }
        }

        let mut arg = None;
        let mut read = SocketRead::new();

        let (buf, n) = loop {
            match read.resume(arg.take()) {
                SocketReadResult::Ok { buf, n } => break (buf, n),
                SocketReadResult::Io { input } => {
                    arg = Some(handle(&mut tcp, input).await.unwrap())
                }
                SocketReadResult::Err { err } => panic!("{err}"),
                SocketReadResult::Eof => panic!("reached unexpected EOF"),
            }
        };

        let mut lines = BufReader::new(&buf[..n]).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            stdout
                .write_all(format!("S: {line}\n").as_bytes())
                .await
                .unwrap();
        }
    }
}

async fn prompt(stdout: &mut Stdout, message: &str) -> String {
    stdout
        .write_all(format!("{message} ").as_bytes())
        .await
        .unwrap();

    stdout.flush().await.unwrap();

    let mut line = String::new();
    BufReader::new(stdin()).read_line(&mut line).await.unwrap();

    line.trim().to_owned()
}
