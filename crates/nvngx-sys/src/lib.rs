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
use libc::wchar_t;

include!("bindings.rs");

pub mod error;
pub use error::*;
