//! Asynchronous stream socket runtime backed by Tokio.

use log::trace;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Result};

use crate::io::{SocketInput, SocketOutput};

/// Processes a [`SocketInput`] request asynchronously using a stream
/// that implements [`AsyncRead`] + [`AsyncWrite`] (e.g.
/// [`tokio::net::TcpStream`] or a Tokio TLS wrapper).
pub async fn handle(
    stream: impl AsyncRead + AsyncWrite + Unpin,
    input: SocketInput,
) -> Result<SocketOutput> {
    match input {
        SocketInput::Read { buf } => read(stream, buf).await,
        SocketInput::Write { buf } => write(stream, buf).await,
    }
}

/// Reads bytes from `stream` into `buf` and returns a
/// [`SocketOutput::Read`] with the number of bytes read.
pub async fn read(mut stream: impl AsyncRead + Unpin, mut buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("reading bytes asynchronously from stream");
    let n = stream.read(&mut buf).await?;
    Ok(SocketOutput::Read { buf, n })
}

/// Writes `buf` to `stream` and returns a [`SocketOutput::Wrote`] with
/// the number of bytes written.
pub async fn write(mut stream: impl AsyncWrite + Unpin, buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("writing bytes asynchronously to stream");
    let n = stream.write(&buf).await?;
    Ok(SocketOutput::Wrote { buf, n })
}
