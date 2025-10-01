//! Vulkan bindings to NGX.
#![cfg(feature = "vk")]

use ash::vk;
use nvngx_sys::{
    vulkan::{
        NVSDK_NGX_ImageViewInfo_VK, NVSDK_NGX_Resource_VK, NVSDK_NGX_Resource_VK_Type,
        NVSDK_NGX_Resource_VK__bindgen_ty_1,
    },
    NVSDK_NGX_Coordinates, NVSDK_NGX_Dimensions, NVSDK_NGX_Feature, NVSDK_NGX_PerfQuality_Value,
    Result,
};

use crate::ngx::{self, Feature, FeatureHandleOps, FeatureOps, FeatureParameterOps};

use super::ngx::{FeatureParameters, SuperSamplingCreateParameters};

pub mod feature;
pub use feature::*;
pub mod super_sampling;
pub use super_sampling::*;
pub mod ray_reconstruction;
pub use ray_reconstruction::*;

fn convert_slice_of_strings_to_cstrings(data: &[String]) -> Result<Vec<std::ffi::CString>> {
    data.iter()
        .cloned()
        .map(std::ffi::CString::new)
        .collect::<Result<_, _>>()
        .map_err(|_| "Couldn't convert the extensions to CStrings.".into())
}

/// Vulkan extensions required for the NVIDIA NGX operation.
#[derive(Debug, Clone)]
pub struct RequiredExtensions {
    /// Vulkan device extensions required for NVIDIA NGX.
    pub device: Vec<String>,
    /// Vulkan instance extensions required for NVIDIA NGX.
    pub instance: Vec<String>,
}

impl RequiredExtensions {
    /// Returns a list of device extensions as a list of
    /// [`std::ffi::CString`].
    pub fn get_device_extensions_c_strings(&self) -> Result<Vec<std::ffi::CString>> {
        convert_slice_of_strings_to_cstrings(&self.device)
    }

    /// Returns a list of instance extensions as a list of
    /// [`std::ffi::CString`].
    pub fn get_instance_extensions_c_strings(&self) -> Result<Vec<std::ffi::CString>> {
        convert_slice_of_strings_to_cstrings(&self.instance)
    }

    /// Returns a list of required vulkan extensions for NGX to work.
    pub fn get() -> Result<Self> {
        let mut instance_extensions: *mut *const std::ffi::c_char = std::ptr::null_mut();
        let mut device_extensions: *mut *const std::ffi::c_char = std::ptr::null_mut();
        let mut instance_count = 0u32;
        let mut device_count = 0u32;
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_RequiredExtensions(
                &mut instance_count,
                &mut instance_extensions,
                &mut device_count,
                &mut device_extensions,
            )
        })?;

        let mut instance = Vec::new();
        for i in 0..instance_count {
            instance.push(unsafe {
                std::ffi::CStr::from_ptr(*instance_extensions.add(i as usize))
                    .to_str()
                    .map(|s| s.to_owned())
                    .unwrap()
            });
        }

        let mut device = Vec::new();
        for i in 0..device_count {
            device.push(unsafe {
                std::ffi::CStr::from_ptr(*device_extensions.add(i as usize))
                    .to_str()
                    .map(|s| s.to_owned())
                    .unwrap()
            });
        }

        // unsafe {
        //     libc::free(device_extensions as _);
        //     libc::free(instance_extensions as _);
        // }

        Ok(Self { device, instance })
    }
}

/// NVIDIA NGX system.
#[repr(transparent)]
#[derive(Debug)]
pub struct System {
    device: vk::Device,
}
impl System {
    /// Creates a new NVIDIA NGX system.
    pub fn new(
        project_id: Option<uuid::Uuid>,
        engine_version: &str,
        application_data_path: &std::path::Path,
        entry: &ash::Entry,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: vk::Device,
    ) -> Result<Self> {
        let engine_type = nvngx_sys::NVSDK_NGX_EngineType::NVSDK_NGX_ENGINE_TYPE_CUSTOM;
        let project_id =
            std::ffi::CString::new(project_id.unwrap_or_else(uuid::Uuid::new_v4).to_string())
                .unwrap();
        let engine_version = std::ffi::CString::new(engine_version).unwrap();
        let application_data_path =
            widestring::WideString::from_str(application_data_path.to_str().unwrap());
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_Init_with_ProjectID(
                project_id.as_ptr(),
                engine_type,
                engine_version.as_ptr(),
                application_data_path.as_ptr().cast(),
                instance.handle(),
                physical_device,
                logical_device,
                entry.static_fn().get_instance_proc_addr,
                instance.fp_v1_0().get_device_proc_addr,
                std::ptr::null(),
                nvngx_sys::NVSDK_NGX_Version::NVSDK_NGX_Version_API,
            )
        })?;

        Ok(Self {
            device: logical_device,
        })
    }

    fn shutdown(&self) -> Result {
        unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_Shutdown1(self.device) }.into()
    }

    /// Creates a new [`Feature`] with the logical device used to create
    /// this [`System`].
    pub fn create_feature<T>(
        &self,
        command_buffer: vk::CommandBuffer,
        feature_type: NVSDK_NGX_Feature,
        parameters: Option<FeatureParameters<T>>,
    ) -> Result<Feature<T>>
    where
        T: FeatureParameterOps
            + FeatureOps<Device = vk::Device, CommandBuffer = vk::CommandBuffer>
            + FeatureHandleOps,
    {
        let parameters = match parameters {
            Some(p) => p,
            None => FeatureParameters::get_capability_parameters()?,
        };
        Feature::new(self.device, command_buffer, feature_type, parameters)
    }

    /// Creates a supersampling (or "DLSS") feature.
    pub fn create_super_sampling_feature<T, P>(
        &self,
        command_buffer: vk::CommandBuffer,
        feature_parameters: FeatureParameters<T>,
        create_parameters: *mut SuperSamplingCreateParameters,
    ) -> Result<ngx::SuperSamplingFeature<T, P>>
    where
        T: FeatureParameterOps
            + FeatureHandleOps
            + FeatureOps<Device = vk::Device, CommandBuffer = vk::CommandBuffer>,
        P: ngx::super_sampling::SuperSamplingEvaluationOps,
    {
        Feature::new_super_sampling(
            self.device,
            command_buffer,
            feature_parameters,
            create_parameters,
        )
    }

    /// Creates a frame generation feature.
    pub fn create_frame_generation_feature<T>(
        &self,
        command_buffer: vk::CommandBuffer,
        feature_parameters: FeatureParameters<T>,
    ) -> Result<Feature<T>>
    where
        T: FeatureHandleOps
            + FeatureOps<Device = vk::Device, CommandBuffer = vk::CommandBuffer>
            + FeatureParameterOps,
    {
        Feature::new_frame_generation(self.device, command_buffer, feature_parameters)
    }

    // TURN BACK ON
    // /// Creates a ray reconstruction feature.
    // pub fn create_ray_reconstruction_feature<T>(
    //     &self,
    //     command_buffer: vk::CommandBuffer,
    //     feature_parameters: FeatureParameters<T>,
    //     create_parameters: RayReconstructionCreateParameters,
    // ) -> Result<RayReconstructionFeature>
    // where
    // T: FeatureHandleOps + FeatureOps + FeatureParameterOps
    // {
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

/// A mode that a vulkan resource might have.
#[derive(Default, Debug, Copy, Clone)]
pub enum VkResourceMode {
    /// Indicates that the resource can only be read.
    #[default]
    Readable,
    /// Indicates that the resource can be written to.
    Writable,
}

/// A struct, objects of which should be hold by
/// [`SuperSamplingEvaluationParameters`] for feature evaluation.
#[derive(Debug, Default, Copy, Clone)]
pub struct VkBufferResourceDescription {
    /// The buffer!
    pub buffer: vk::Buffer,
    /// The size of the buffer in bytes.
    pub size_in_bytes: usize,
    /// The mode this resource has.
    pub mode: VkResourceMode,
}

/// A struct, objects of which should be hold by
/// [`SuperSamplingEvaluationParameters`] for feature evaluation.
#[derive(Debug, Default, Copy, Clone)]
pub struct VkImageResourceDescription {
    /// The image view.
    pub image_view: vk::ImageView,
    /// The image.
    pub image: vk::Image,
    /// The subresource range.
    pub subresource_range: vk::ImageSubresourceRange,
    /// The format.
    pub format: vk::Format,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
    /// The mode this resource has.
    pub mode: VkResourceMode,
}

impl VkImageResourceDescription {
    /// Sets the writable bit.
    pub fn set_writable(&mut self) {
        self.mode = VkResourceMode::Writable;
    }
}

impl From<VkImageResourceDescription> for NVSDK_NGX_Resource_VK {
    fn from(value: VkImageResourceDescription) -> Self {
        let vk_image_subresource_range = vk::ImageSubresourceRange {
            aspect_mask: value.subresource_range.aspect_mask,
            base_mip_level: value.subresource_range.base_mip_level,
            base_array_layer: value.subresource_range.base_array_layer,
            level_count: value.subresource_range.level_count,
            layer_count: value.subresource_range.layer_count,
        };

        let image_view_info = NVSDK_NGX_ImageViewInfo_VK {
            ImageView: value.image_view,
            Image: value.image,
            SubresourceRange: vk_image_subresource_range,
            Format: value.format,
            Width: value.width,
            Height: value.height,
        };

        // Cannot use a Rust `union` constructor because bindgen doesn't know
        // our `Vk*` types anymore and wraps them in __BindgenUnionField:
        // https://github.com/rust-lang/rust-bindgen/issues/2187#issuecomment-3048892937
        let mut image_resource = NVSDK_NGX_Resource_VK__bindgen_ty_1::default();
        unsafe { *image_resource.ImageViewInfo.as_mut() = image_view_info }

        Self {
            Resource: image_resource,
            Type: NVSDK_NGX_Resource_VK_Type::NVSDK_NGX_RESOURCE_VK_TYPE_VK_IMAGEVIEW,
            ReadWrite: matches!(value.mode, VkResourceMode::Writable),
        }
    }
}

// #[derive(Debug)]
// pub struct FeatureCommonInfo {
//     path_list_info:,

// }

// /// Contains information common to all features, used by NGX in
// /// determining requested feature availability.
// #[derive(Debug, Clone)]
// pub struct FeatureDiscoveryBuilder {
//     /// API Struct version number.
//     sdk_version: Option<bindings::NVSDK_NGX_Version>,
//     /// Valid NVSDK_NGX_Feature enum corresponding to DLSS v3 Feature
//     /// which is being queried for availability.
//     feature_type: Option<bindings::NVSDK_NGX_Feature>,
//     /// Unique Id provided by NVIDIA corresponding to a particular
//     /// Application or alternatively custom Id set by Engine.
//     application_identifier: Option<bindings::NVSDK_NGX_Application_Identifier>,
//     /// Folder to store logs and other temporary files (write access
//     /// required), normally this would be a location in Documents or
//     /// ProgramData.
//     application_data_path: Option<widestring::WideCString>,
//     /// Contains information common to all features, presently only a
//     /// list of all paths feature dlls can be located in, other than the
//     /// default path - application directory.
//     common_info: Option<FeatureCommonInfo>,
// }

// impl FeatureDiscoveryBuilder {
//     /// Creates a new feature discovery builder. The created feature
//     /// discovery builder contains blanket values.
//     pub fn new() -> Self {
//         Self(bindings::NVSDK_NGX_FeatureDiscoveryInfo {
//             SDKVersion: bindings::NVSDK_NGX_Version::NVSDK_NGX_Version_API,
//             FeatureID: bindings::NVSDK_NGX_Feature::NVSDK_NGX_Feature_Reserved_Unknown,

//         })
//     }

//     /// Consumes the builder and obtains the requirements for the
//     /// requested feature based on the information provided.
//     pub fn get_requirements(self) -> Result<FeatureRequirement> {
//         unimplemented!()
//     }
// }

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn features() {
        // TODO: initialise vulkan and be able to do this.
        // dbg!(super::FeatureParameters::get_capability_parameters().unwrap());
    }

    #[test]
    fn get_required_extensions() {
        assert!(super::RequiredExtensions::get().is_ok());
    }

    /// Ignored as it just needs to compile.
    #[test]
    #[ignore]
    fn insert_parameter_debug_macro() -> super::Result {
        let mut map = HashMap::new();
        let parameters =
            super::FeatureParameters::<crate::vk::VulkanPlatform>::get_capability_parameters()
                .unwrap();
        crate::insert_parameter_debug!(
            map,
            parameters,
            (nvngx_sys::NVSDK_NGX_EParameter_Reserved00, i32),
            (
                nvngx_sys::NVSDK_NGX_EParameter_SuperSampling_Available,
                bool
            ),
            (nvngx_sys::NVSDK_NGX_EParameter_InPainting_Available, bool),
            (
                nvngx_sys::NVSDK_NGX_EParameter_ImageSuperResolution_Available,
                bool
            ),
        );

        Ok(())
    }
}
