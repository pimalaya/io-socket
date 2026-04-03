//! Collection of I/O-free, resumable and composable socket state
//! machines.
//!
//! Coroutines emit [`SocketInput`] requests that need to be processed
//! by [runtimes] in order to continue their progression.
//!
//! [`SocketInput`]: crate::io::SocketInput
//! [runtimes]: crate::runtimes

pub mod read;
pub mod read_exact;
pub mod read_to_end;
pub mod write;
