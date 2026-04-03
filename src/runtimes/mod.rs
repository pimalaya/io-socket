//! Collection of socket runtimes.
//!
//! A runtime contains all the I/O logic, and is responsible for
//! processing [`SocketInput`] requests emitted by [coroutines] and
//! returning the corresponding [`SocketOutput`].
//!
//! If you miss a runtime matching your requirements, you can easily
//! implement your own by taking example on the existing ones. PRs are
//! welcomed!
//!
//! [`SocketInput`]: crate::io::SocketInput
//! [`SocketOutput`]: crate::io::SocketOutput
//! [coroutines]: crate::coroutines

#[cfg(feature = "std-stream")]
pub mod std_stream;
#[cfg(feature = "std-udp-socket")]
pub mod std_udp_socket;
#[cfg(feature = "tokio-stream")]
pub mod tokio_stream;
