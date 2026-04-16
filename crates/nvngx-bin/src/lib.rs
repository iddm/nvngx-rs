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

/// Target platform for DLSS libraries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
}

impl Platform {
    /// Returns the platform matching the Cargo `CARGO_CFG_TARGET_OS` environment variable.
    ///
    /// Cargo sets this for build scripts. Panics outside that context or on
    /// unsupported targets.
    pub fn for_current_target() -> Self {
        let os = std::env::var("CARGO_CFG_TARGET_OS")
            .expect("CARGO_CFG_TARGET_OS not set (are you in a build script?)");
        match os.as_str() {
            "linux" => Self::Linux,
            "windows" => Self::Windows,
            _ => panic!("unsupported target OS {os:?} — only Linux and Windows have DLSS binaries"),
        }
    }
}

/// Build configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    Release,
    Dev,
}

/// DLSS feature variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// DLSS Super Resolution.
    Dlss,
    /// DLSS Ray Reconstruction.
    Dlssd,
    /// DLSS Frame Generation.
    Dlssg,
}

/// Returns the path to a DLSS shared library for the given feature, platform, and configuration.
pub fn dlss_path(feature: Feature, platform: Platform, config: Config) -> PathBuf {
    let config_dir = match config {
        Config::Release => "rel",
        Config::Dev => "dev",
    };

    let filename = match (feature, platform) {
        (Feature::Dlss, Platform::Linux) => {
            format!("libnvidia-ngx-dlss.so.{}", dlss_version())
        }
        (Feature::Dlssd, Platform::Linux) => {
            format!("libnvidia-ngx-dlssd.so.{}", dlss_version())
        }
        (Feature::Dlssg, Platform::Linux) => {
            format!("libnvidia-ngx-dlssg.so.{}", dlss_version())
        }
        (Feature::Dlss, Platform::Windows) => "nvngx_dlss.dll".into(),
        (Feature::Dlssd, Platform::Windows) => "nvngx_dlssd.dll".into(),
        (Feature::Dlssg, Platform::Windows) => "nvngx_dlssg.dll".into(),
    };

    let platform_dir = match platform {
        Platform::Linux => "Linux_x86_64",
        Platform::Windows => "Windows_x86_64",
    };

    Path::new(DLSS_LIB_DIR)
        .join(platform_dir)
        .join(config_dir)
        .join(filename)
}
