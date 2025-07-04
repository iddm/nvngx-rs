//! The NGX-rs is a crate carefully wrapping the NVIDIA NGX library,
//! providing some abstractions in order to make the use easier.
#![deny(missing_docs)]

pub mod vk;
pub use vk::*;

pub use nvngx_sys as sys;
