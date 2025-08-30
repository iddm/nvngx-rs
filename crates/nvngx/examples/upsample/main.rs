mod vk_mini_init;

use ash::vk;

fn main() {
    env_logger::init();

    let required_extensions = nvngx::vk::RequiredExtensions::get().unwrap();
    let physical_device_features2 = vk::PhysicalDeviceFeatures2::default();

    let vk_mini_init = vk_mini_init::VkMiniInit::new(
        required_extensions.instance.clone(),
        required_extensions.device.clone(),
        &physical_device_features2
    );

    let system = nvngx::System::new(
        None,
        env!("CARGO_PKG_VERSION"),
        &std::env::current_dir().unwrap(), // Run with the __NGX_LOG_LEVEL=1 environment variable to see more logs from NGX (Linux Only)
        &vk_mini_init.entry_fn,
        &vk_mini_init.instance,
        vk_mini_init.physical_device,
        vk_mini_init.device.handle(),
    ).unwrap();

    let capability_parameters = nvngx::vk::FeatureParameters::get_capability_parameters().unwrap();

    let supported = capability_parameters.supports_super_sampling().is_ok();
    assert!(supported, "DLSS not supported on this device");
    //     Ok(req) => util::print_extensions(&req.instance, &req.device),
    //     Err(e) => {
    //         eprintln!("Failed to query required extensions: {}", e);
    //         std::process::exit(1);
    //     }
    // }
}
