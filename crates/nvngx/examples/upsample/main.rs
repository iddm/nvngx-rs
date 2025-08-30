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

    let _system = nvngx::System::new(
        None,
        env!("CARGO_PKG_VERSION"),
        &std::env::current_dir().unwrap(), // Run with the __NGX_LOG_LEVEL=1 environment variable to see more logs from NGX (Linux Only)
        &vk_mini_init.entry_fn,
        &vk_mini_init.instance,
        vk_mini_init.physical_device,
        vk_mini_init.device.handle(),
    )
    .unwrap();

    let capability_parameters = nvngx::vk::FeatureParameters::get_capability_parameters().unwrap();
    assert!(
        capability_parameters.supports_super_sampling().is_ok(),
        "DLSS not supported on this device"
    );

    // ---
    // Demo: copy host RGBA buffer -> device image, then image -> host buffer and save as PNG
    // ---

    let device = &vk_mini_init.device;
    // Command submission is handled by helper now

    let mut allocator = vk_mini_init.get_allocator();

    // 1) Load source pixels
    let (src_rgba, src_width, src_height) = allocations::load_png_rgba8(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/upsample/baboon.png"),
    );
    // let (dst_width, dst_height) = (src_width * 2, src_height * 2);

    // 2) Create host-visible staging buffer and upload
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

    // 3) Create device-local image (optimal tiling)
    let device_color_input_image = allocations::create_image_optimal(
        device,
        &mut allocator,
        src_width,
        src_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
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

    // 4) Create readback buffer
    let readback = allocations::create_buffer(
        device,
        &mut allocator,
        src_rgba.len() as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
        "readback",
    );

    // 5-6) Record, submit and wait using helper
    vk_mini_init
        .record_and_submit(|cmd, dev| {
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
                    cmd,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            // Clear motion vectors to (0,0)
            let clear_mv = vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] };
            unsafe {
                dev.cmd_clear_color_image(
                    cmd,
                    device_motion_vectors_image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &clear_mv,
                    std::slice::from_ref(&subresource),
                );
            }

            // Clear depth to 1.0 (far)
            let clear_depth = vk::ClearColorValue { float32: [1.0, 0.0, 0.0, 0.0] };
            unsafe {
                dev.cmd_clear_color_image(
                    cmd,
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
                .image_extent(vk::Extent3D { width: src_width, height: src_height, depth: 1 });
            unsafe {
                dev.cmd_copy_buffer_to_image(
                    cmd,
                    staging.buffer,
                    device_color_input_image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&region),
                );
            }

            // Transition image: TRANSFER_DST_OPTIMAL -> TRANSFER_SRC_OPTIMAL
            let barrier_to_src = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .image(device_color_input_image.image)
                .subresource_range(subresource)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ);
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
            unsafe {
                let barriers = [barrier_to_src, mv_barrier_to_read, depth_barrier_to_read];
                dev.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER | vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                )
            };

            // Copy image -> buffer (readback)
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
                .image_extent(vk::Extent3D { width: src_width, height: src_height, depth: 1 });
            unsafe {
                dev.cmd_copy_image_to_buffer(
                    cmd,
                    device_color_input_image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    readback.buffer,
                    std::slice::from_ref(&img_to_buf_region),
                );
            }
        })
        .unwrap();

    // 7) Read back and save
    let mapped = readback
        .allocation
        .mapped_slice()
        .expect("readback mapped");

    image::save_buffer_with_format(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/upsample/roundtrip.png"),
        &mapped,
        src_width,
        src_height,
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

}
