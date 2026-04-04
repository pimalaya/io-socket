//! I/O-free coroutine to read exactly N bytes from a socket.

use alloc::vec::Vec;
use core::mem;

use log::{debug, trace};
use thiserror::Error;

use crate::{
    coroutines::read::{ReadSocket, ReadSocketError, ReadSocketResult},
    io::{SocketInput, SocketOutput},
};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Error)]
pub enum ReadSocketExactError {
    /// The socket reached EOF before `max` bytes could be read.
    ///
    /// Carries the number of bytes still missing, the total target, and
    /// the bytes that were accumulated before EOF.
    #[error("Unexpected EOF, expected to read {0}/{1} more bytes")]
    UnexpectedEof(usize, usize, Vec<u8>),

    /// Error from the inner [`ReadSocket`] coroutine.
    #[error(transparent)]
    Read(#[from] ReadSocketError),
}

/// Output emitted after the coroutine finishes its progression.
#[derive(Clone, Debug)]
pub enum ReadSocketExactResult {
    /// The coroutine has successfully read exactly the requested number
    /// of bytes.
    Ok { buf: Vec<u8> },

    /// A socket I/O needs to be performed to make the coroutine
    /// progress.
    Io { input: SocketInput },

    /// An error occurred during the coroutine progression.
    Err { err: ReadSocketExactError },
}

/// I/O-free coroutine to read exactly N bytes from a socket.
///
/// Drives a [`ReadSocket`] coroutine in a loop, accumulating chunks
/// until exactly `max` bytes have been received. If the socket reaches
/// EOF before that, the coroutine returns
/// [`ReadSocketExactError::UnexpectedEof`] along with whatever bytes
/// were accumulated.
#[derive(Debug)]
pub struct ReadSocketExact {
    /// Inner single-read coroutine, reused across iterations.
    read: ReadSocket,

    /// Accumulates bytes across multiple reads until `max` is reached.
    buffer: Vec<u8>,

    /// Target byte count.
    max: usize,
}

impl ReadSocketExact {
    /// Creates a new coroutine using a read buffer with
    /// [`ReadSocket::DEFAULT_CAPACITY`] capacity.
    pub fn new(max: usize) -> Self {
        Self::with_capacity(ReadSocket::DEFAULT_CAPACITY, max)
    }

    /// Creates a new coroutine using a read buffer with the given
    /// capacity.
    ///
    /// The inner read buffer is capped at `max` — there is no point
    /// requesting more bytes than the remaining target.
    pub fn with_capacity(capacity: usize, max: usize) -> Self {
        trace!("init coroutine to read exactly {max} bytes (capacity: {capacity})");
        let read = ReadSocket::with_capacity(capacity.min(max));
        let buffer = Vec::with_capacity(max);
        Self { read, buffer, max }
    }

    /// Pre-fills the accumulation buffer with `bytes`.
    ///
    /// Useful when bytes have already been read by a previous coroutine
    /// and need to count toward the target.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    /// Advances the coroutine.
    ///
    /// Pass `None` on the first call. On subsequent calls, pass the
    /// [`SocketOutput`] returned by the runtime after processing the
    /// last emitted [`SocketInput`].
    pub fn resume(&mut self, mut arg: Option<SocketOutput>) -> ReadSocketExactResult {
        loop {
            if self.buffer.len() >= self.max {
                let buf = mem::take(&mut self.buffer);
                break ReadSocketExactResult::Ok { buf };
            }

            let remaining = self.max - self.buffer.len();
            debug!("{remaining} remaining bytes to read");

            // Shrink the inner read buffer so we never overshoot the target.
            if remaining < self.read.capacity() {
                self.read.truncate(remaining);
            }

            match self.read.resume(arg.take()) {
                ReadSocketResult::Ok { buf, n } => {
                    self.buffer.extend_from_slice(&buf[..n]);
                    self.read.replace(buf);
                }
                ReadSocketResult::Err { err } => {
                    break ReadSocketExactResult::Err { err: err.into() };
                }
                ReadSocketResult::Io { input } => {
                    break ReadSocketExactResult::Io { input };
                }
                ReadSocketResult::Eof => {
                    let buf = mem::take(&mut self.buffer);
                    let err = ReadSocketExactError::UnexpectedEof(remaining, self.max, buf);
                    break ReadSocketExactResult::Err { err };
                }
            }
        }
    }
}
