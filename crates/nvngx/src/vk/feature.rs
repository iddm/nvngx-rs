//! Describes NGX features and their parameters.
use crate::ngx::{FeatureHandleOps, FeatureOps, FeatureParameterOps};
use ash::vk;

/// Holds backend data for featurehandle, featureparameters and feature
#[derive(Debug, Clone, Copy)]
pub struct VulkanPlatform;

impl FeatureHandleOps for VulkanPlatform {
    fn release_handle(
        handle: *mut nvngx_sys::NVSDK_NGX_Handle,
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_ReleaseFeature(handle) }.into()
    }
}

impl FeatureParameterOps for VulkanPlatform {
    fn create_parameters(
    ) -> nvngx_sys::Result<*mut nvngx_sys::NVSDK_NGX_Parameter, nvngx_sys::Error> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        let res: nvngx_sys::Result<(), nvngx_sys::Error> =
            unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_AllocateParameters(&mut ptr) }.into();
        res.map(|_| ptr)
    }

    fn get_capability_parameters(
    ) -> nvngx_sys::Result<*mut nvngx_sys::NVSDK_NGX_Parameter, nvngx_sys::Error> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        let res: nvngx_sys::Result<(), nvngx_sys::Error> =
            unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut ptr) }.into();
        res.map(|_| ptr)
    }

    fn release_parameters(
        params: *mut nvngx_sys::NVSDK_NGX_Parameter,
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_DestroyParameters(params) }.into()
    }
}

impl FeatureOps for VulkanPlatform {
    type Device = vk::Device;
    type CommandBuffer = vk::CommandBuffer;

    fn create_feature(
        device: Self::Device,
        command_buffer: Self::CommandBuffer,
        feature_type: nvngx_sys::NVSDK_NGX_Feature,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
        handle: &mut *mut nvngx_sys::NVSDK_NGX_Handle,
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_CreateFeature1(
                device,
                command_buffer,
                feature_type,
                parameters,
                handle,
            )
        }
        .into()
    }

    fn create_super_sampling_feature(
        device: Self::Device,
        command_buffer: Self::CommandBuffer,
        handle: &mut *mut nvngx_sys::NVSDK_NGX_Handle,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
        create_params: *mut crate::ngx::super_sampling::SuperSamplingCreateParameters, // Platform-specific create params
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe {
            nvngx_sys::vulkan::HELPERS_NGX_VULKAN_CREATE_DLSS_EXT1(
                device,
                command_buffer,
                1,
                1,
                handle,
                parameters,
                create_params as *mut _,
            )
        }
        .into()
    }

    fn create_ray_reconstruction_feature(
        device: Self::Device,
        command_buffer: Self::CommandBuffer,
        handle: &mut *mut nvngx_sys::NVSDK_NGX_Handle,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
        create_params: *mut u8, // Platform-specific create params
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe {
            nvngx_sys::vulkan::HELPERS_NGX_VULKAN_CREATE_DLSSD_EXT1(
                device,
                command_buffer,
                1,
                1,
                handle,
                parameters,
                create_params as *mut _,
            )
        }
        .into()
    }

    fn get_scratch_buffer_size(
        feature_type: nvngx_sys::NVSDK_NGX_Feature,
        parameters: *const nvngx_sys::NVSDK_NGX_Parameter,
    ) -> nvngx_sys::Result<usize, nvngx_sys::Error> {
        let mut size = 0usize;
        let res: nvngx_sys::Result<(), nvngx_sys::Error> = unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_GetScratchBufferSize(
                feature_type,
                parameters,
                &mut size,
            )
        }
        .into();
        res.map(|_| size)
    }

    fn evaluate_feature(
        command_buffer: Self::CommandBuffer,
        handle: *mut nvngx_sys::NVSDK_NGX_Handle,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
    ) -> nvngx_sys::Result<(), nvngx_sys::Error> {
        unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_EvaluateFeature_C(
                command_buffer,
                handle,
                parameters,
                Some(feature_progress_callback),
            )
        }
        .into()
    }
}

unsafe extern "C" fn feature_progress_callback(progress: f32, _should_cancel: *mut bool) {
    log::debug!("Feature evalution progress={progress}.");
}

/// Feature used outside the crate
pub type VulkanFeature = crate::ngx::feature::Feature<VulkanPlatform>;
/// Parameters used outside the crate
pub type VulkanFeatureParameters = crate::ngx::feature::FeatureParameters<VulkanPlatform>;
/// Handle used outside the crate
pub type VulkanFeatureHandle = crate::ngx::feature::FeatureHandle<VulkanPlatform>;

// impl VulkanFeature {
//     /// Creates a new Vulkan Feature
//     pub fn new_vulkan(
//         device: vk::Device,
//         command_buffer: vk::CommandBuffer,
//         feature_type: NVSDK_NGX_Feature,
//         parameters: VulkanFeatureParameters,
//     ) -> Result<Self, nvngx_sys::Error> {
//         Self::new(device, command_buffer, feature_type, parameters)
//     }

//     /// Creates a new SuperSampling feature for Vulkan
//     pub fn new_super_sampling_vulkan(
//         device: vk::Device,
//         command_buffer: vk::CommandBuffer,
//         parameters: VulkanFeatureParameters,
//         create_parameters: &mut [u8],
//     ) -> Result<crate::ngx::super_sampling::SuperSamplingFeature<VulkanPlatform, VulkanPlatform>, nvngx_sys::Error>
//     where
//         VulkanPlatform: crate::ngx::super_sampling::SuperSamplingEvaluationOps,
//     {
//         Self::new_super_sampling(device, command_buffer, parameters, create_parameters)
//     }
// }
