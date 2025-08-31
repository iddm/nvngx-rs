mod allocations;
mod imgops;
mod vk_mini_init;

use ash::vk;
use image::ColorType;
use nvngx::SuperSamplingFeature;

fn main() {
    let required_extensions = nvngx::vk::RequiredExtensions::get().unwrap();
    let mut vulkan_12_features =
        vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);
    let mut vulkan_13_features =
        vk::PhysicalDeviceVulkan13Features::default().synchronization2(true);
    let physical_device_features2 = vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut vulkan_12_features)
        .push_next(&mut vulkan_13_features);

    let vk_mini_init = vk_mini_init::VkMiniInit::new(
        required_extensions.instance.clone(),
        required_extensions.device.clone(),
        &physical_device_features2,
    );

    let system = nvngx::System::new(
        None,
        env!("CARGO_PKG_VERSION"),
        &std::env::current_dir().unwrap(), // Run with the __NGX_LOG_LEVEL=1 environment variable to see more logs from NGX (Linux Only)
        &vk_mini_init.entry_fn,
        &vk_mini_init.instance,
        vk_mini_init.physical_device,
        vk_mini_init.device.handle(),
    )
    .unwrap();

    // 1) Load source pixels
    let (src_rgba, src_width, src_height) = allocations::load_png_rgba8(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/upsample/baboon.png"
    ));
    let (dst_width, dst_height) = (src_width * 2, src_height * 2);

    // 2) --- Create DLSS feature ---
    let capability_parameters =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("capability params");
    assert!(
        capability_parameters.supports_super_sampling().is_ok(),
        "DLSS not supported on this device"
    );
    let create_params = nvngx::vk::SuperSamplingCreateParameters::new(
        src_width,
        src_height,
        dst_width,
        dst_height,
        Some(nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_Balanced),
        None,
    );
    
    let mut ss: nvngx_sys::Result<SuperSamplingFeature> =
        Err(nvngx::sys::Error::Other("Not initialized".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            ss = system.create_super_sampling_feature(cb, capability_parameters, create_params);
        })
        .unwrap();
    let mut ss = ss.expect("create DLSS feature");

    // 3) Create host-visible staging buffer for uploading the image, and resources for the DLSS input/output
    let mut allocator = vk_mini_init.get_allocator();

    let mut staging = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        src_rgba.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    staging
        .allocation
        .mapped_slice_mut()
        .expect("staging mapped")
        .copy_from_slice(src_rgba.as_raw());

    let mut color_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut mv_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R16G16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut depth_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut out_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        dst_width,
        dst_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );
    let readback = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        (dst_width * dst_height * 4) as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );

    // 5) Record the image upload, DLSS evaluation and readback
    vk_mini_init
        .record_and_submit(|cb, dev| {
            // Prepare images for upload/clear and DLSS
            color_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );
            mv_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );
            depth_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );

            // Clear and upload inputs
            imgops::copy_buffer_to_image(
                dev,
                cb,
                staging.buffer,
                color_img.image,
                src_width,
                src_height,
            );
            imgops::clear_color_image(dev, cb, mv_img.image, [0.0, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, depth_img.image, [1.0, 0.0, 0.0, 0.0]);

            // Transition inputs for sampling and output for storage
            color_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_READ,
                vk::ImageLayout::GENERAL,
            );
            mv_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_READ,
                vk::ImageLayout::GENERAL,
            );
            depth_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_READ,
                vk::ImageLayout::GENERAL,
            );
            out_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );

            // Fill evaluation params
            let subresource = imgops::default_subresource_range();
            let out_desc = nvngx::vk::VkImageResourceDescription {
                image_view: out_img.view,
                image: out_img.image,
                subresource_range: subresource,
                format: vk::Format::R8G8B8A8_UNORM,
                width: dst_width,
                height: dst_height,
                mode: nvngx::vk::VkResourceMode::Writable,
            };
            let color_desc = nvngx::vk::VkImageResourceDescription {
                image_view: color_img.view,
                image: color_img.image,
                subresource_range: subresource,
                format: vk::Format::R8G8B8A8_UNORM,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };
            let mv_desc = nvngx::vk::VkImageResourceDescription {
                image_view: mv_img.view,
                image: mv_img.image,
                subresource_range: subresource,
                format: vk::Format::R16G16_SFLOAT,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };
            let depth_desc = nvngx::vk::VkImageResourceDescription {
                image_view: depth_img.view,
                image: depth_img.image,
                subresource_range: subresource,
                format: vk::Format::R32_SFLOAT,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };

            // Hook them up
            let eval = ss.get_evaluation_parameters_mut();
            eval.set_color_input(color_desc);
            eval.set_color_output(out_desc);
            eval.set_motions_vectors(mv_desc, None);
            eval.set_depth_buffer(depth_desc);
            eval.set_jitter_offsets(0.0, 0.0);
            eval.set_reset(true);
            eval.set_rendering_dimensions([0, 0], [src_width, src_height]);

            // Evaluate
            ss.evaluate(cb).expect("DLSS evaluate");

            // Prepare output for readback and copy
            out_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                out_img.image,
                readback.buffer,
                dst_width,
                dst_height,
            );
        })
        .unwrap();

    // 7) Host read back and save
    let mapped = readback.allocation.mapped_slice().expect("readback mapped");

    image::save_buffer_with_format(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/upsample/upsampled.png"
        ),
        &mapped,
        dst_width,
        dst_height,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("save png");

    // Cleanup GPU allocations
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, staging);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, color_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, mv_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, depth_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, out_img);
}
