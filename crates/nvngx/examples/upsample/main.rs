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

    let optimal_settings = nvngx::vk::SuperSamplingOptimalSettings::get_optimal_settings(
        &capability_parameters,
        1024,
        1024,
        nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_MaxPerf, // Scales 2x
    )
    .unwrap();
    assert!(optimal_settings.render_height == 512 && optimal_settings.render_width == 512);

    
    // ---
    // Demo: copy host RGBA buffer -> device image, then image -> host buffer and save as PNG
    // ---

    let device = &vk_mini_init.device;
    let queue = vk_mini_init.queue;
    let queue_family_index = vk_mini_init.queue_family_index;

    let mut allocator = vk_mini_init.get_allocator();

    // 1) Load source pixels
    let (src_rgba, width, height) = allocations::load_png_rgba8(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/upsample/baboon.png"),
    );
    let byte_size = (width as u64) * (height as u64) * 4;

    // 2) Create host-visible staging buffer and upload
    let mut staging = allocations::create_buffer(
        device,
        &mut allocator,
        byte_size,
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
    let device_image = allocations::create_image_rgba8_optimal(
        device,
        &mut allocator,
        width,
        height,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
        "device-image",
    );

    // 4) Create readback buffer
    let readback = allocations::create_buffer(
        device,
        &mut allocator,
        byte_size,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
        "readback",
    );

    // 5) Command pool + buffer
    let cmd_pool_ci = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    let cmd_pool = unsafe { device.create_command_pool(&cmd_pool_ci, None) }.unwrap();
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(cmd_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = unsafe { device.allocate_command_buffers(&alloc_info) }.unwrap()[0];

    // 6) Record
    let begin_info = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(cmd, &begin_info) }.unwrap();

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
        .image(device_image.image)
        .subresource_range(subresource)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier_to_dst),
        )
    };

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
            width,
            height,
            depth: 1,
        });
    unsafe {
        device.cmd_copy_buffer_to_image(
            cmd,
            staging.buffer,
            device_image.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        );
    }

    // Transition image: TRANSFER_DST_OPTIMAL -> TRANSFER_SRC_OPTIMAL
    let barrier_to_src = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(device_image.image)
        .subresource_range(subresource)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::TRANSFER_READ);
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier_to_src),
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
        .image_extent(vk::Extent3D { width, height, depth: 1 });
    unsafe {
        device.cmd_copy_image_to_buffer(
            cmd,
            device_image.image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            readback.buffer,
            std::slice::from_ref(&img_to_buf_region),
        );
    }

    // Make transfer writes visible to host
    let buf_barrier = vk::BufferMemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::HOST_READ)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(readback.buffer)
        .offset(0)
        .size(byte_size);
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            &[],
            std::slice::from_ref(&buf_barrier),
            &[],
        )
    };

    unsafe { device.end_command_buffer(cmd) }.unwrap();

    let submit_info = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd));
    unsafe { 
        device.queue_submit(queue, std::slice::from_ref(&submit_info), vk::Fence::null()).unwrap();
        device.device_wait_idle().unwrap();
        device.destroy_command_pool(cmd_pool, None);
    }

    // 7) Read back and save
    let mapped = readback
        .allocation
        .mapped_slice()
        .expect("readback mapped");

    image::save_buffer_with_format(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/upsample/roundtrip.png"),
        &mapped,
        width,
        height,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("save png");

    // Cleanup GPU allocations
    allocations::destroy_buffer(device, &mut allocator, staging);
    allocations::destroy_buffer(device, &mut allocator, readback);
    allocations::destroy_image(device, &mut allocator, device_image);

}
