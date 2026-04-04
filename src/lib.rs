#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![no_std]
extern crate alloc;
// Runtime modules that wrap std or tokio bring std back explicitly.
#[cfg(any(
    feature = "std-stream",
    feature = "std-udp-socket",
    feature = "tokio-stream"
))]
extern crate std;

pub mod coroutines;
pub mod io;
pub mod runtimes;
