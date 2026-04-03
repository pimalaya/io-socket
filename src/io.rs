//! Socket input and output.

use std::fmt;

/// Socket input emitted by [coroutines] and processed by [runtimes].
///
/// Represents all the possible operations that a socket coroutine can
/// ask for. Runtimes must handle all variants and return a matching
/// [`SocketOutput`].
///
/// [coroutines]: crate::coroutines
/// [runtimes]: crate::runtimes
#[derive(Clone, Eq, PartialEq)]
pub enum SocketInput {
    /// Request to read bytes from the socket into the provided
    /// buffer.
    Read { buf: Vec<u8> },

    /// Request to write the provided bytes into the socket.
    Write { buf: Vec<u8> },
}

impl fmt::Debug for SocketInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { buf, .. } => write!(f, "SocketInput::Read({})", buf.len()),
            Self::Write { buf, .. } => write!(f, "SocketInput::Write({})", buf.len()),
        }
    }
}

/// Socket output returned by [runtimes] after processing a
/// [`SocketInput`].
///
/// Each variant corresponds to the matching [`SocketInput`] variant
/// and carries the original buffer back alongside the byte count
/// actually transferred.
///
/// [runtimes]: crate::runtimes
#[derive(Clone, Eq, PartialEq)]
pub enum SocketOutput {
    /// Response to a [`SocketInput::Read`] request.
    Read {
        /// The buffer that was filled.
        buf: Vec<u8>,

        /// The number of bytes read. 0 usually means that the socket
        /// has reached the end of file (EOF).
        n: usize,
    },

    /// Response to a [`SocketInput::Write`] request.
    ///
    /// `buf` is the buffer that was written; `n` is the number of bytes
    /// actually sent. When `n == 0`, the socket has reached end-of-file.
    Wrote {
        /// The buffer that was written.
        buf: Vec<u8>,

        /// The amount of bytes that have been written. 0 usually
        /// means that the socket has reached the end of file (EOF),
        /// and n > buf.len() means that write is not complete.
        n: usize,
    },
}

impl fmt::Debug for SocketOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { buf, n } => write!(f, "SocketOutput::Read({n}/{})", buf.len()),
            Self::Wrote { buf, n } => write!(f, "SocketOutput::Wrote({n}/{})", buf.len()),
        }
    }
}
