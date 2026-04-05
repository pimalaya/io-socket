//! I/O-free coroutine to read from a socket until EOF.

use alloc::vec::Vec;
use core::mem;

use log::trace;
use thiserror::Error;

use crate::{
    coroutines::read::{SocketRead, SocketReadError, SocketReadResult},
    io::{SocketInput, SocketOutput},
};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Error)]
pub enum SocketReadToEndError {
    /// Error from the inner [`SocketRead`] coroutine.
    #[error(transparent)]
    Read(#[from] SocketReadError),
}

/// Output emitted after the coroutine finishes its progression.
#[derive(Clone, Debug)]
pub enum SocketReadToEndResult {
    /// The coroutine has successfully read all bytes up to EOF.
    Ok { buf: Vec<u8> },

    /// A socket I/O needs to be performed to make the coroutine
    /// progress.
    Io { input: SocketInput },

    /// An error occurred during the coroutine progression.
    Err { err: SocketReadToEndError },
}

/// I/O-free coroutine to read from a socket until EOF.
///
/// Drives a [`SocketRead`] coroutine in a loop, accumulating every
/// chunk into an internal buffer. Returns [`SocketReadToEndResult::Ok`]
/// when the socket signals EOF (`n == 0`).
#[derive(Debug)]
pub struct SocketReadToEnd {
    /// Inner single-read coroutine, reused across iterations.
    read: SocketRead,

    /// Accumulates all bytes read until EOF.
    buffer: Vec<u8>,
}

impl SocketReadToEnd {
    /// Creates a new coroutine using a read buffer with
    /// [`SocketRead::DEFAULT_CAPACITY`] capacity.
    pub fn new() -> Self {
        Self::with_capacity(SocketRead::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine using a read buffer with the given
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        trace!("init coroutine to read until EOF (capacity: {capacity})");
        let read = SocketRead::with_capacity(capacity);
        let buffer = Vec::with_capacity(capacity);
        Self { read, buffer }
    }

    /// Pre-fills the accumulation buffer with `bytes`.
    ///
    /// Useful when bytes have already been read by a previous coroutine
    /// and should be included in the final result.
    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.buffer.extend(bytes);
    }

    /// Advances the coroutine.
    ///
    /// Pass `None` on the first call. On subsequent calls, pass the
    /// [`SocketOutput`] returned by the runtime after processing the
    /// last emitted [`SocketInput`].
    pub fn resume(&mut self, mut arg: Option<SocketOutput>) -> SocketReadToEndResult {
        loop {
            match self.read.resume(arg.take()) {
                SocketReadResult::Ok { buf, n } => {
                    self.buffer.extend_from_slice(&buf[..n]);
                    self.read.replace(buf);
                }
                SocketReadResult::Err { err } => {
                    break SocketReadToEndResult::Err { err: err.into() };
                }
                SocketReadResult::Io { input } => {
                    break SocketReadToEndResult::Io { input };
                }
                SocketReadResult::Eof => {
                    let buf = mem::take(&mut self.buffer);
                    break SocketReadToEndResult::Ok { buf };
                }
            }
        }
    }
}

impl Default for SocketReadToEnd {
    fn default() -> Self {
        Self::new()
    }
}
