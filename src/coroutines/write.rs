//! I/O-free coroutine to write bytes into a buffer.

use alloc::vec::Vec;
use core::{fmt, mem};

use log::{debug, trace};
use thiserror::Error;

use crate::io::{SocketInput, SocketOutput};

/// Errors that can occur during the coroutine progression.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum SocketWriteError {
    /// The coroutine received an invalid argument.
    ///
    /// Occurs when the coroutine receives an I/O response from
    /// another coroutine, which should not happen if the runtime maps
    /// correctly the arguments.
    #[error("Expected argument SocketOutput::Write, got {0:?}")]
    UnexpectedArg(SocketOutput),
}

/// Output emitted after a coroutine finishes its progression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocketWriteResult {
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
    Err { err: SocketWriteError },
}

/// I/O-free coroutine to write bytes into a buffer.
#[derive(Clone, Eq, PartialEq)]
pub struct SocketWrite {
    buf: Vec<u8>,
}

impl SocketWrite {
    /// Creates a new coroutine that will write the given bytes to the
    /// socket.
    pub fn new(buf: Vec<u8>) -> Self {
        trace!("init coroutine for writing {} bytes", buf.len());
        Self { buf }
    }

    /// Makes the write progress.
    pub fn resume(&mut self, arg: Option<SocketOutput>) -> SocketWriteResult {
        let Some(arg) = arg else {
            trace!("wants to write bytes");
            let buf = mem::take(&mut self.buf);
            let input = SocketInput::Write { buf };
            return SocketWriteResult::Io { input };
        };

        trace!("resume after writing bytes");
        let SocketOutput::Wrote { buf, n } = arg else {
            let err = SocketWriteError::UnexpectedArg(arg);
            return SocketWriteResult::Err { err };
        };

        if n == 0 {
            debug!("received EOF");
            return SocketWriteResult::Eof;
        }

        debug!("wrote {n}/{} bytes", buf.capacity());
        SocketWriteResult::Ok { buf, n }
    }
}

impl fmt::Debug for SocketWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SocketWrite({})", self.buf.len())
    }
}
