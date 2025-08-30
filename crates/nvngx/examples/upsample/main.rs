mod allocations;
mod vk_mini_init;

use ash::vk;
use image::ColorType;

fn main() {
    let required_extensions = nvngx::vk::RequiredExtensions::get().unwrap();
    let physical_device_features2 = vk::PhysicalDeviceFeatures2::default();

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

    // 2) --- DLSS create + evaluate ---
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

    let device = &vk_mini_init.device;
    // Command submission is handled by helper now

    let mut allocator = vk_mini_init.get_allocator();

    // 3) Create host-visible staging buffer and upload
    let mut staging = allocations::create_buffer(
        device,
        &mut allocator,
        src_rgba.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
        "staging",
    );
    staging
        .allocation
        .mapped_slice_mut()
        .expect("staging mapped")
        .copy_from_slice(src_rgba.as_raw());

    // 4) Create device-local image (optimal tiling)
    let device_color_input_image = allocations::create_image_optimal(
        device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        "device-image",
    );

    // 3b) Create motion vector and depth images on device
    let device_motion_vectors_image = allocations::create_image_optimal(
        device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R16G16_SFLOAT, // RG16 float motion vectors
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        "motion-vectors",
    );
    let device_depth_image = allocations::create_image_optimal(
        device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R32_SFLOAT, // R32 float linear depth
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        "depth-image",
    );

    // 3c) Create an output image (DLSS upscaled target). We'll upscale 2x for demo.
    let (dst_width, dst_height) = (src_width * 2, src_height * 2);
    let device_dlss_output_image = allocations::create_image_optimal(
        device,
        &mut allocator,
        dst_width,
        dst_height,
        vk::Format::R8G8B8A8_UNORM,
        // DLSS writes via storage, we'll also copy to buffer later
        vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
        "dlss-output",
    );

    // Create image views for all images we pass to DLSS
    let subresource_range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    let mk_view = |img: vk::Image, fmt: vk::Format| -> vk::ImageView {
        let view_ci = vk::ImageViewCreateInfo::default()
            .image(img)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(fmt)
            .subresource_range(subresource_range);
        unsafe {
            device
                .create_image_view(&view_ci, None)
                .expect("create image view")
        }
    };
    let color_view = mk_view(device_color_input_image.image, vk::Format::R8G8B8A8_UNORM);
    let mv_view = mk_view(device_motion_vectors_image.image, vk::Format::R16G16_SFLOAT);
    let depth_view = mk_view(device_depth_image.image, vk::Format::R32_SFLOAT);
    let output_view = mk_view(device_dlss_output_image.image, vk::Format::R8G8B8A8_UNORM);

    // 4) Create readback buffer sized for upscaled output
    let readback = allocations::create_buffer(
        device,
        &mut allocator,
        (dst_width as usize * dst_height as usize * 4) as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
        "readback",
    );

    // 5-6) Record, submit and wait using helper
    vk_mini_init
        .record_and_submit(|cb, dev| {
            // Transition image: UNDEFINED -> TRANSFER_DST_OPTIMAL
            let subresource = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };
            let barrier_to_dst = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(device_color_input_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            // Barriers for motion vectors and depth images as well
            let mv_barrier_to_dst = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(device_motion_vectors_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            let depth_barrier_to_dst = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(device_depth_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            unsafe {
                let barriers = [barrier_to_dst, mv_barrier_to_dst, depth_barrier_to_dst];
                dev.cmd_pipeline_barrier(
                    cb,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            // Clear motion vectors to (0,0)
            let clear_mv = vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            };
            unsafe {
                dev.cmd_clear_color_image(
                    cb,
                    device_motion_vectors_image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &clear_mv,
                    std::slice::from_ref(&subresource),
                );
            }

            // Clear depth to 1.0 (far)
            let clear_depth = vk::ClearColorValue {
                float32: [1.0, 0.0, 0.0, 0.0],
            };
            unsafe {
                dev.cmd_clear_color_image(
                    cb,
                    device_depth_image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &clear_depth,
                    std::slice::from_ref(&subresource),
                );
            }

            // Copy buffer -> image
            let region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: src_width,
                    height: src_height,
                    depth: 1,
                });
            unsafe {
                dev.cmd_copy_buffer_to_image(
                    cb,
                    staging.buffer,
                    device_color_input_image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&region),
                );
            }

            // Transition image: TRANSFER_DST_OPTIMAL -> TRANSFER_SRC_OPTIMAL
            let barrier_to_src = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(device_color_input_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);
            // Transition the newly cleared images to SHADER_READ_ONLY_OPTIMAL
            let mv_barrier_to_read = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(device_motion_vectors_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);
            let depth_barrier_to_read = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(device_depth_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);
            // Prepare DLSS output image for storage writes
            let output_barrier_to_general = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(device_dlss_output_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::SHADER_WRITE);
            unsafe {
                let barriers = [
                    barrier_to_src,
                    mv_barrier_to_read,
                    depth_barrier_to_read,
                    output_barrier_to_general,
                ];
                dev.cmd_pipeline_barrier(
                    cb,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER
                        | vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            let mut ss = system
                .create_super_sampling_feature(cb, capability_parameters, create_params)
                .expect("create DLSS feature");

            // Fill evaluation params
            let eval = ss.get_evaluation_parameters_mut();
            let out_desc = nvngx::vk::VkImageResourceDescription {
                image_view: output_view,
                image: device_dlss_output_image.image,
                subresource_range: subresource,
                format: vk::Format::R8G8B8A8_UNORM,
                width: dst_width,
                height: dst_height,
                mode: nvngx::vk::VkResourceMode::Writable,
            };
            let color_desc = nvngx::vk::VkImageResourceDescription {
                image_view: color_view,
                image: device_color_input_image.image,
                subresource_range: subresource,
                format: vk::Format::R8G8B8A8_UNORM,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };
            let mv_desc = nvngx::vk::VkImageResourceDescription {
                image_view: mv_view,
                image: device_motion_vectors_image.image,
                subresource_range: subresource,
                format: vk::Format::R16G16_SFLOAT,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };
            let depth_desc = nvngx::vk::VkImageResourceDescription {
                image_view: depth_view,
                image: device_depth_image.image,
                subresource_range: subresource,
                format: vk::Format::R32_SFLOAT,
                width: src_width,
                height: src_height,
                mode: nvngx::vk::VkResourceMode::Readable,
            };

            // Hook them up
            eval.set_color_input(color_desc);
            eval.set_color_output(out_desc);
            eval.set_motions_vectors(mv_desc, None);
            eval.set_depth_buffer(depth_desc);
            eval.set_jitter_offsets(0.0, 0.0);
            eval.set_reset(true);
            eval.set_rendering_dimensions([0, 0], [src_width, src_height]);

            // Evaluate
            ss.evaluate(cb).expect("DLSS evaluate");

            // Barrier output GENERAL -> TRANSFER_SRC for readback
            let out_to_copy = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(device_dlss_output_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ);
            unsafe {
                dev.cmd_pipeline_barrier(
                    cb,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    std::slice::from_ref(&out_to_copy),
                );
            }

            // Copy output image -> buffer (readback)
            let img_to_buf_region = vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: dst_width,
                    height: dst_height,
                    depth: 1,
                });
            unsafe {
                dev.cmd_copy_image_to_buffer(
                    cb,
                    device_dlss_output_image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    readback.buffer,
                    std::slice::from_ref(&img_to_buf_region),
                );
            }
        })
        .unwrap();

    // 7) Read back and save
    let mapped = readback.allocation.mapped_slice().expect("readback mapped");

    image::save_buffer_with_format(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/upsample/roundtrip.png"
        ),
        &mapped,
        dst_width,
        dst_height,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("save png");

    // Cleanup GPU allocations
    allocations::destroy_buffer(device, &mut allocator, staging);
    allocations::destroy_buffer(device, &mut allocator, readback);
    allocations::destroy_image(device, &mut allocator, device_color_input_image);
    allocations::destroy_image(device, &mut allocator, device_motion_vectors_image);
    allocations::destroy_image(device, &mut allocator, device_depth_image);
    allocations::destroy_image(device, &mut allocator, device_dlss_output_image);

    // Destroy image views
    unsafe {
        device.destroy_image_view(color_view, None);
        device.destroy_image_view(mv_view, None);
        device.destroy_image_view(depth_view, None);
        device.destroy_image_view(output_view, None);
    }
}
