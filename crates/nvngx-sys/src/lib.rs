//! `nvngx-sys` provides low-level "sys" bindings to the NVIDIA NGX library.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

pub mod ngx {
    use libc::wchar_t;
    include!("ngx_bindings.rs");
}
pub use ngx::*;

pub mod error;
pub use error::*;

pub mod vulkan;

pub mod directx;
