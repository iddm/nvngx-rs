//! Prebuilt NVIDIA NGX (DLSS) shared libraries.
//!
//! This crate provides path accessors for the prebuilt DLSS shared libraries
//! bundled in the DLSS SDK submodule. Applications use these paths to locate
//! and copy the DLSS inference DLLs at build or deploy time.
//!
//! # Warning: git/path dependency only
//!
//! This crate resolves the DLSS binaries via a relative path into the
//! `nvngx-sys/DLSS` submodule (`CARGO_MANIFEST_DIR/../nvngx-sys/DLSS/lib`).
//! This **only works as a git or path dependency** — it cannot be published
//! to crates.io (the binaries are 170 MB+ of proprietary NVIDIA blobs that
//! exceed the 10 MB crate size limit, and cannot be downloaded at build time
//! from a public URL).

use std::path::{Path, PathBuf};

const DLSS_LIB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../nvngx-sys/DLSS/lib");

fn dlss_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
        .split_once("+v")
        .expect("CARGO_PKG_VERSION must contain +v<dlss-version> build metadata")
        .1
}

/// Returns the path to the DLSS Super Resolution shared library (Linux release).
pub fn dlss_so_path_linux() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join(format!(
        "Linux_x86_64/rel/libnvidia-ngx-dlss.so.{}",
        dlss_version()
    ))
}

/// Returns the path to the DLSS Ray Reconstruction shared library (Linux release).
pub fn dlssd_so_path_linux() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join(format!(
        "Linux_x86_64/rel/libnvidia-ngx-dlssd.so.{}",
        dlss_version()
    ))
}

/// Returns the path to the DLSS Frame Generation shared library (Linux release).
pub fn dlssg_so_path_linux() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join(format!(
        "Linux_x86_64/rel/libnvidia-ngx-dlssg.so.{}",
        dlss_version()
    ))
}

/// Returns the path to the DLSS Super Resolution DLL (Windows release).
pub fn dlss_dll_path_windows() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Windows_x86_64/rel/nvngx_dlss.dll")
}

/// Returns the path to the DLSS Ray Reconstruction DLL (Windows release).
pub fn dlssd_dll_path_windows() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Windows_x86_64/rel/nvngx_dlssd.dll")
}

/// Returns the path to the DLSS Frame Generation DLL (Windows release).
pub fn dlssg_dll_path_windows() -> PathBuf {
    Path::new(DLSS_LIB_DIR).join("Windows_x86_64/rel/nvngx_dlssg.dll")
}
