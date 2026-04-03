//! Synchronous UDP datagram socket runtime backed by [`UdpSocket`].

use std::{io::Result, net::UdpSocket};

use log::trace;

use crate::io::{SocketInput, SocketOutput};

/// Processes a [`SocketInput`] request synchronously using a connected
/// [`UdpSocket`].
///
/// The socket must have been connected with [`UdpSocket::connect`]
/// beforehand so that `send`/`recv` know the peer address.
pub fn handle(socket: &mut UdpSocket, input: SocketInput) -> Result<SocketOutput> {
    match input {
        SocketInput::Read { buf } => recv(socket, buf),
        SocketInput::Write { buf } => send(socket, buf),
    }
}

/// Receives a datagram from the connected peer into `buf` and returns a
/// [`SocketOutput::Read`] with the number of bytes received.
pub fn recv(socket: &mut UdpSocket, mut buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("receiving bytes synchronously from UDP socket");
    let n = socket.recv(&mut buf)?;
    Ok(SocketOutput::Read { buf, n })
}

/// Sends `buf` to the connected peer and returns a
/// [`SocketOutput::Wrote`] with the number of bytes sent.
pub fn send(socket: &mut UdpSocket, buf: Vec<u8>) -> Result<SocketOutput> {
    trace!("sending bytes synchronously on UDP socket");
    let n = socket.send(&buf)?;
    Ok(SocketOutput::Wrote { buf, n })
}
