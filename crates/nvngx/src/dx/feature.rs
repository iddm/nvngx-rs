//! Directx features and parameters

use crate::ngx::FeatureHandleOps;

/// Holds backend data for featurehandle, featureparameters and feature
#[derive(Debug, Clone, Copy)]
pub struct DX12Platform;

impl FeatureHandleOps for DX12Platform {
    fn release_handle(
        _handle: *mut nvngx_sys::NVSDK_NGX_Handle,
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        todo!()
    }
}
