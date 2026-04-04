//! Synchronous stream socket runtime backed by [`std::io`].

use alloc::vec::Vec;
use std::io::{Read, Result, Write};

use log::trace;

use crate::io::{SocketInput, SocketOutput};

/// Processes a [`SocketInput`] request synchronously using a stream
/// that implements [`Read`] + [`Write`].
///
/// For example [`std::net::TcpStream`], a TLS wrapper, or a Unix
/// stream socket.
pub fn handle(stream: impl Read + Write, input: SocketInput) -> Result<SocketOutput> {
    match input {
        SocketInput::Read { buf } => read(stream, buf),
        SocketInput::Write { buf } => write(stream, buf),
    }
}

/// Reads bytes from `stream` into `buf` and returns a
/// [`SocketOutput::Read`] with the number of bytes read.
pub fn read(mut stream: impl Read, mut buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("reading bytes synchronously from stream");
    let n = stream.read(&mut buf)?;
    Ok(SocketOutput::Read { buf, n })
}

/// Writes `buf` to `stream` and returns a [`SocketOutput::Wrote`]
/// with the number of bytes written.
pub fn write(mut stream: impl Write, buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("writing bytes synchronously to stream");
    let n = stream.write(&buf)?;
    Ok(SocketOutput::Wrote { buf, n })
}
