//! Common information provided to NGX at system initialization time.
//!
//! These types mirror [`nvngx_sys::NVSDK_NGX_FeatureCommonInfo`] and friends from the NGX
//! SDK. They are API-agnostic (Vulkan / D3D12 / CUDA), though only the Vulkan
//! path currently exposes them via [`crate::vk::System::new`].

use std::ffi::CStr;
use std::path::Path;

use nvngx_sys::{NVSDK_NGX_Feature, NVSDK_NGX_Logging_Level};

/// Controls how verbose NGX's own logging is.
#[derive(Debug, Clone, Copy)]
pub enum LoggingLevel {
    /// Disable NGX logging.
    Off,
    /// Standard information and error logging.
    On,
    /// Verbose logging (diagnostic details).
    Verbose,
}

impl From<LoggingLevel> for NVSDK_NGX_Logging_Level {
    fn from(l: LoggingLevel) -> Self {
        match l {
            LoggingLevel::Off => Self::NVSDK_NGX_LOGGING_LEVEL_OFF,
            LoggingLevel::On => Self::NVSDK_NGX_LOGGING_LEVEL_ON,
            LoggingLevel::Verbose => Self::NVSDK_NGX_LOGGING_LEVEL_VERBOSE,
        }
    }
}

/// Opt-in routing of NGX log lines through the [`log`] crate.
///
/// When supplied via [`FeatureCommonInfo::logging`], every NGX log message is
/// passed to [`log::log!`] at a level derived from the NGX level
/// ([`LoggingLevel::Verbose`] → [`log::Level::Debug`], [`LoggingLevel::On`] →
/// [`log::Level::Info`]). This avoids having to enable NGX's registry-based
/// log sinks on Windows.
#[derive(Debug, Clone, Copy)]
pub struct LoggingConfig {
    /// Minimum NGX severity to forward. NGX honours the *higher* of this value
    /// and whatever is configured via the registry.
    pub minimum_level: LoggingLevel,
    /// When [`true`], NGX won't write to its other log sinks (files, on-screen
    /// console). Log lines are delivered *only* through the [`log`] crate.
    pub disable_other_sinks: bool,
}

/// Common information provided to NGX at initialization time.
///
/// This mirrors [`nvngx_sys::NVSDK_NGX_FeatureCommonInfo`] from the NGX SDK, exposing:
/// - an optional list of additional search paths for feature DLLs
///   (e.g. `nvngx_dlss.dll`), searched in descending order of preference in
///   addition to the application folder;
/// - an optional [`LoggingConfig`] that pipes NGX logs through the [`log`] crate.
#[derive(Debug, Clone, Copy, Default)]
pub struct FeatureCommonInfo<'a> {
    /// Extra directories to search for feature DLLs, in descending order of
    /// preference. The application folder is always searched as well.
    pub search_paths: &'a [&'a Path],
    /// If [`Some`], NGX logs are routed through the [`log`] crate.
    pub logging: Option<LoggingConfig>,
}

/// Static `extern "C"` callback that pipes NGX log lines into the [`log`]
/// crate. NGX calls this from arbitrary threads; [`log`]'s macros are
/// thread-safe so no additional synchronization is needed.
pub(crate) unsafe extern "C" fn log_crate_callback(
    message: *const std::os::raw::c_char,
    level: NVSDK_NGX_Logging_Level,
    source: NVSDK_NGX_Feature,
) {
    let log_level = match level {
        NVSDK_NGX_Logging_Level::NVSDK_NGX_LOGGING_LEVEL_VERBOSE => log::Level::Debug,
        NVSDK_NGX_Logging_Level::NVSDK_NGX_LOGGING_LEVEL_ON => log::Level::Info,
        _ => return,
    };
    let msg = if message.is_null() {
        std::borrow::Cow::Borrowed("<null>")
    } else {
        unsafe { CStr::from_ptr(message) }.to_string_lossy()
    };
    log::log!(target: "nvngx", log_level, "[{source:?}] {}", msg.trim_end());
}
