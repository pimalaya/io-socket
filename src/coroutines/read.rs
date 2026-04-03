//! I/O-free coroutine to read bytes into a buffer.

use std::{fmt, mem};

use log::{debug, trace};
use thiserror::Error;

use crate::io::{SocketInput, SocketOutput};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ReadSocketError {
    /// The coroutine received an invalid argument.
    ///
    /// Occurs when the coroutine receives an I/O response from
    /// another coroutine, which should not happen if the runtime maps
    /// correctly the arguments.
    #[error("Expected argument SocketOutput::Read, got {0:?}")]
    UnexpectedArg(SocketOutput),
}

/// Output emitted after a coroutine finishes its progression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadSocketResult {
    /// The coroutine has successfully terminated its progression.
    Ok { buf: Vec<u8>, n: usize },

    /// A socket I/O needs to be performed to make the coroutine
    /// progress.
    Io { input: SocketInput },

    /// The coroutine reached the End Of File.
    ///
    /// Only the consumer can determine if its an error or not.
    Eof,

    /// An error occurred during the coroutine progression.
    Err { err: ReadSocketError },
}

/// I/O-free coroutine to read bytes into a buffer.
#[derive(Clone, Eq, PartialEq)]
pub struct ReadSocket {
    buf: Vec<u8>,
}

impl ReadSocket {
    /// The default read buffer capacity.
    pub const DEFAULT_CAPACITY: usize = 8 * 1024;

    /// Creates a new coroutine to read bytes using a buffer with
    /// [`Self::DEFAULT_CAPACITY`] capacity.
    ///
    /// See [`Self::with_capacity`] for a custom buffer capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates a new coroutine to read bytes using a buffer with the
    /// given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        trace!("init coroutine to read bytes (capacity: {capacity})");
        let buf = vec![0; capacity];
        Self { buf }
    }

    /// Returns the buffer capacity.
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Shortens the buffer to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.buf.truncate(len);
        self.buf.shrink_to(len);
    }

    /// Replaces the inner buffer with the given one.
    pub fn replace(&mut self, mut buf: Vec<u8>) {
        buf.fill(0);
        self.buf = buf;
    }

    /// Makes the read progress.
    pub fn resume(&mut self, arg: Option<SocketOutput>) -> ReadSocketResult {
        let Some(arg) = arg else {
            trace!("wants to read bytes");
            let mut buf = vec![0; self.buf.capacity()];
            mem::swap(&mut buf, &mut self.buf);
            let input = SocketInput::Read { buf };
            return ReadSocketResult::Io { input };
        };

        trace!("resume after reading bytes");
        let SocketOutput::Read { buf, n } = arg else {
            let err = ReadSocketError::UnexpectedArg(arg);
            return ReadSocketResult::Err { err };
        };

        if n == 0 {
            debug!("received EOF");
            return ReadSocketResult::Eof;
        }

        debug!("read {n}/{} bytes", buf.capacity());
        ReadSocketResult::Ok { buf, n }
    }
}

impl fmt::Debug for ReadSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReadSocket({})", self.buf.len())
    }
}

impl Default for ReadSocket {
    fn default() -> Self {
        Self::new()
    }
}
