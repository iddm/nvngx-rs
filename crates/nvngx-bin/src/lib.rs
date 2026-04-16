//! Prebuilt NVIDIA NGX (DLSS) shared libraries.
//!
//! This crate provides path accessors for the prebuilt DLSS shared libraries
//! bundled in the DLSS SDK submodule. Applications use these paths to locate
//! and copy the DLSS inference DLLs at build or deploy time.
//!
//! Paths are resolved relative to this crate's source directory via
//! [`env!("CARGO_MANIFEST_DIR")`], so they are valid at compile time of any
//! crate that depends on `nvngx-bin` (including build scripts).

use std::path::{Path, PathBuf};

const DLSS_LIB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../nvngx-sys/DLSS/lib");

/// Returns the path to the DLSS Super Resolution shared library (Linux release).
pub fn dlss_so_path_linux() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Linux_x86_64/rel/libnvidia-ngx-dlss.so.310.1.0")
}

/// Returns the path to the DLSS Ray Reconstruction shared library (Linux release).
pub fn dlssd_so_path_linux() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Linux_x86_64/rel/libnvidia-ngx-dlssd.so.310.1.0")
}

/// Returns the path to the DLSS Super Resolution DLL (Windows release).
pub fn dlss_dll_path_windows() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Windows_x86_64/rel/nvngx_dlss.dll")
}

/// Returns the path to the DLSS Ray Reconstruction DLL (Windows release).
pub fn dlssd_dll_path_windows() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Windows_x86_64/rel/nvngx_dlssd.dll")
}
