//! The errors the NGX crate might have.

use crate as bindings;

/// The result type used within the crate.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// The error type.
#[derive(Debug, Clone)]
pub enum Error {
    /// An internal NVIDIA NGX error, not related to the crate.
    Internal(bindings::NVSDK_NGX_Result),
    /// Any other error which doesn't originate from the NVIDIA NGX.
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Internal(code) => {
                format!("Internal error: code={code}")
            }
            Self::Other(s) => format!("Other error: {s}"),
        })
    }
}

impl From<bindings::NVSDK_NGX_Result> for Error {
    fn from(value: bindings::NVSDK_NGX_Result) -> Self {
        Self::Internal(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(value: &'a str) -> Self {
        Self::Other(value.to_owned())
    }
}

impl From<bindings::NVSDK_NGX_Result> for Result {
    fn from(value: bindings::NVSDK_NGX_Result) -> Self {
        match value {
            bindings::NVSDK_NGX_Result::NVSDK_NGX_Result_Success => Ok(()),
            code => Err(Error::Internal(code)),
        }
    }
}

impl std::fmt::Display for bindings::NVSDK_NGX_Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chars = unsafe { bindings::GetNGXResultAsString(*self as _) };
        let length = unsafe { libc::wcslen(chars) };
        let string = unsafe { widestring::WideCString::from_ptr(chars.cast(), length) }
            .map_err(|_| std::fmt::Error)?;
        let string = string.to_string().map_err(|_| std::fmt::Error)?;
        f.write_str(&string)?;
        // unsafe {
        //     libc::free(chars as *mut _);
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate as bindings;

    #[test]
    fn test_error_message() {
        let string =
            bindings::NVSDK_NGX_Result::NVSDK_NGX_Result_FAIL_FeatureNotSupported.to_string();
        assert_eq!(string, "NVSDK_NGX_Result_FAIL_FeatureNotSupported");
    }
}
