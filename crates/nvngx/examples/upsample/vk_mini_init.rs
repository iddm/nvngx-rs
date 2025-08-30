use std::ffi::{c_char, CStr, CString};

use ash::vk;
use log::{info, warn};

pub struct VkMiniInit {
    pub entry_fn: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue_family_index: u32,
    pub queue: vk::Queue,
}

impl VkMiniInit {
    pub fn new(
        mut instance_extensions: Vec<String>,
        mut device_extensions: Vec<String>,
        desired_physical_device_features2: &vk::PhysicalDeviceFeatures2,
    ) -> Self {
        #[cfg(debug_assertions)]
        instance_extensions.push("VK_EXT_debug_utils".to_owned());

        let entry_fn = unsafe { ash::Entry::load().unwrap() };

        // Handle validation layers and settings based on debug build and runtime flag
        let (layer_names, instance) = {
            if Self::should_enable_validation() {
                instance_extensions.push("VK_EXT_layer_settings".to_owned());
                let layer_names = vec!["VK_LAYER_KHRONOS_validation".to_owned()];

                // TODO: ash is broken and incorrectly sets the value count, watch for issue https://github.com/ash-rs/ash/issues/985
                // need to set the value count manually
                let mut layer_settings_base = vk::LayerSettingEXT::default()
                    .layer_name(c"VK_LAYER_KHRONOS_validation")
                    .ty(vk::LayerSettingTypeEXT::BOOL32);
                layer_settings_base.value_count = 1;

                let sliced_true = vk::TRUE.to_le_bytes();

                let mut x1 = layer_settings_base.setting_name(c"printf_enable");
                x1.p_values = sliced_true.as_ptr().cast();

                let mut x2 = layer_settings_base.setting_name(c"validate_sync");
                x2.p_values = sliced_true.as_ptr().cast();

                let mut x3 = layer_settings_base.setting_name(c"gpuav_enable");
                x3.p_values = sliced_true.as_ptr().cast();

                let mut x4 = layer_settings_base.setting_name(c"validate_best_practices");
                x4.p_values = sliced_true.as_ptr().cast();

                let settings = [x1, x2, x3, x4];
                let mut pnext = vk::LayerSettingsCreateInfoEXT::default().settings(&settings);

                let instance =
                    Self::create_instance(&entry_fn, &mut pnext, &instance_extensions, &layer_names);
                (layer_names, instance)
            } else {
                let layer_names: Vec<String> = Vec::new();
                let mut pnext = vk::LayerSettingsCreateInfoEXT::default();
                let instance =
                    Self::create_instance(&entry_fn, &mut pnext, &instance_extensions, &layer_names);
                (layer_names, instance)
            }
        };

        let desired_device_extensions_cptr = Self::get_cptr_vec_from_str_slice(&device_extensions);

        let (physical_device, device) = unsafe {
            instance
                .enumerate_physical_devices()
                .unwrap()
                .iter()
                .copied()
                .find_map(|physical_device| {
                    let queues_create_info = vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(0)
                        .queue_priorities(&[1.0]);
                    let mut device_create_info = vk::DeviceCreateInfo::default()
                        .queue_create_infos(std::slice::from_ref(&queues_create_info))
                        .enabled_extension_names(&desired_device_extensions_cptr.0);
                    device_create_info.p_next =
                        desired_physical_device_features2 as *const _ as *const std::ffi::c_void;

                    let gpu_name = {
                        let mut physical_device_properties =
                            vk::PhysicalDeviceProperties2::default();
                        instance.get_physical_device_properties2(
                            physical_device,
                            &mut physical_device_properties,
                        );
                        CStr::from_ptr(physical_device_properties.properties.device_name.as_ptr())
                    };

                    match instance.create_device(physical_device, &device_create_info, None) {
                        Ok(device) => {
                            info!(
                                "Device created successfully for {gpu_name:?} with extensions {device_extensions:?}",
                            );
                            Some((physical_device, device))
                        }
                        Err(e) => {
                            info!(
                                "Device creation failed for {gpu_name:?} with error: {e}"
                            );
                            None
                        }
                    }
                })
                .expect("Could not find suitable physical device")
        };

        VkMiniInit {
            entry_fn,
            physical_device,
            instance,
            queue_family_index: 0,
            queue: unsafe { device.get_device_queue(0, 0) },
            device,
        }
    }

    fn create_instance<T: vk::ExtendsInstanceCreateInfo + ?Sized>(
        entry_fn: &ash::Entry,
        instance_pnext: &mut T,
        desired_instance_extensions: &[String],
        desired_layer_names: &[String],
    ) -> ash::Instance {
        let application_info = vk::ApplicationInfo::default()
            .application_name(c"NVNGX Sample")
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(c"Custom")
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::make_api_version(0, 1, 3, 0));

        let cstr_layer_names = Self::get_cptr_vec_from_str_slice(desired_layer_names);
        let cstr_extension_names = Self::get_cptr_vec_from_str_slice(desired_instance_extensions);

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&cstr_layer_names.0)
            .enabled_extension_names(&cstr_extension_names.0)
            .push_next(instance_pnext);

        unsafe {
            entry_fn
                .create_instance(&instance_create_info, None)
                .expect("Could not create VkInstance")
        }
    }

    fn should_enable_validation() -> bool {
        #[cfg(debug_assertions)]
        return true;

        #[cfg(not(debug_assertions))]
        false
    }

    fn get_cptr_vec_from_str_slice(input: &[String]) -> (Vec<*const c_char>, Vec<CString>) {
        let input_cstr_vec: Vec<CString> =
            input.iter().cloned().map(|s| CString::new(s).unwrap()).collect();
        let input_cptr_vec = input_cstr_vec.iter().map(|s| s.as_ptr()).collect();
        (input_cptr_vec, input_cstr_vec)
    }
}

impl Drop for VkMiniInit {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}