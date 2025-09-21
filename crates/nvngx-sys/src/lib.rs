//! `nvngx-sys` provides low-level "sys" bindings to the NVIDIA NGX library.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

use ash::vk::{
    Buffer as VkBuffer, CommandBuffer as VkCommandBuffer, Device as VkDevice,
    ExtensionProperties as VkExtensionProperties, Format as VkFormat, Image as VkImage,
    ImageSubresourceRange as VkImageSubresourceRange, ImageView as VkImageView,
    Instance as VkInstance, PFN_vkGetDeviceProcAddr, PFN_vkGetInstanceProcAddr,
    PhysicalDevice as VkPhysicalDevice,
};

include!("bindings.rs");

pub mod error;
pub use error::*;

/// The correct way to implement [`Default`] for this type, as bindgen
/// does not generate the proper default values for the
/// inline-initialised members. We provide the [`Default`]
/// implementation manually.
impl Default for NVSDK_NGX_DLSSG_Opt_Eval_Params {
    fn default() -> Self {
        Self {
            multiFrameCount: 1,
            multiFrameIndex: 1,
            minRelativeLinearDepthObjectSeparation: 40.0f32,
            ..unsafe { ::std::mem::zeroed() }
        }
    }
}
