//! Directx bindings for NVNGX
#![cfg(all(windows, feature = "dx"))]

use nvngx_sys::{NVSDK_NGX_Feature, Result};
use windows::{
    core::Interface as _,
    Win32::Graphics::Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList},
};

pub mod feature;
pub use feature::*;

// use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use crate::ngx::{Feature, FeatureHandleOps, FeatureOps, FeatureParameterOps, FeatureParameters, SuperSamplingCreateParameters};

pub mod super_sampling;
pub use super_sampling::*;

/// API the application is using.
#[derive(Debug)]
pub enum GraphicsAPI {
    /// Vulkan API
    Vulkan,
    /// Directx12 API
    Directx,
}

/// NVIDIA NGX system.
#[repr(transparent)]
#[derive(Debug)]
pub struct System {
    device: ID3D12Device,
}

impl System {
    /// Creates a new NVIDIA NGX system.
    pub fn new(
        project_id: Option<uuid::Uuid>,
        engine_version: &str,
        application_data_path: &std::path::Path,
        device: &ID3D12Device,
    ) -> Result<Self> {
        let engine_type = nvngx_sys::NVSDK_NGX_EngineType::NVSDK_NGX_ENGINE_TYPE_CUSTOM;
        let project_id =
            std::ffi::CString::new(project_id.unwrap_or_else(uuid::Uuid::new_v4).to_string())
                .unwrap();
        let engine_version = std::ffi::CString::new(engine_version).unwrap();
        let application_data_path =
            widestring::WideString::from_str(application_data_path.to_str().unwrap());
        #[allow(clippy::missing_transmute_annotations)] // Transmutes will be removed soon again"
        Result::from(unsafe {
            nvngx_sys::directx::NVSDK_NGX_D3D12_Init_with_ProjectID(
                project_id.as_ptr(),
                engine_type,
                engine_version.as_ptr(),
                application_data_path.as_ptr().cast(),
                device.as_raw().cast(),
                std::ptr::null(),
                nvngx_sys::NVSDK_NGX_Version::NVSDK_NGX_Version_API,
            )
        })?;

        Ok(Self {
            device: device.clone(),
        })
    }

    fn shutdown(&self) -> Result {
        unsafe { nvngx_sys::directx::NVSDK_NGX_D3D12_Shutdown1(self.device.as_raw().cast()) }.into()
    }

    /// Creates a new [`Feature`] with the logical device used to create
    /// this [`System`].
    pub fn create_feature<T>(
        &self,
        command_buffer: &ID3D12GraphicsCommandList,
        feature_type: NVSDK_NGX_Feature,
        parameters: Option<FeatureParameters<T>>,
    ) -> Result<Feature<T>> 
     where
     T: FeatureParameterOps + FeatureOps<Device = (), CommandBuffer = ID3D12GraphicsCommandList> + FeatureHandleOps,
    {
        let parameters = match parameters {
            Some(p) => p,
            None => FeatureParameters::get_capability_parameters()?,
        };
        Feature::new((), command_buffer.clone(), feature_type, parameters) // TODO device needs to be optional
    }

    /// Creates a supersampling (or "DLSS") feature.
    pub fn create_super_sampling_feature<T, P>(
        &self,
        command_buffer: &ID3D12GraphicsCommandList,
        feature_parameters: FeatureParameters<T>,
        create_parameters: *mut SuperSamplingCreateParameters,
    ) -> Result<crate::ngx::SuperSamplingFeature<T, P>>
    where
     T: FeatureParameterOps + FeatureOps<Device = (), CommandBuffer = ID3D12GraphicsCommandList> + FeatureHandleOps,
     P: crate::ngx::super_sampling::SuperSamplingEvaluationOps
     {
        Feature::new_super_sampling((), command_buffer.clone(), feature_parameters, create_parameters) // TODO device needs to be optional.
    }

    /// Creates a frame generation feature.
    pub fn create_frame_generation_feature<T>(
        &self,
        command_buffer: &ID3D12GraphicsCommandList,
        feature_parameters: FeatureParameters<T>,
    ) -> Result<Feature<T>> 
     where
     T: FeatureParameterOps + FeatureOps<Device = (), CommandBuffer = ID3D12GraphicsCommandList> + FeatureHandleOps,
    {
        Feature::new_frame_generation((), command_buffer.clone(), feature_parameters) // Also device
    }

    // TODO: implement ray reconstruction for dx12
    // /// Creates a ray reconstruction feature.
    // pub fn create_ray_reconstruction_feature(
    //     &self,
    //     command_buffer: vk::CommandBuffer,
    //     feature_parameters: FeatureParameters,
    //     create_parameters: RayReconstructionCreateParameters,
    // ) -> Result<RayReconstructionFeature> {
    //     Feature::new_ray_reconstruction(
    //         self.device,
    //         command_buffer,
    //         feature_parameters,
    //         create_parameters,
    //     )
    // }
}

impl Drop for System {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown() {
            log::error!("Couldn't shutdown the NGX system {self:?}: {e}");
        }
    }
}
