//! Frame Generation (DLSSG) example.

#[path = "../common/mod.rs"]
mod common;
use common::{allocations, imgops, vk_mini_init};

use ash::vk;
use image::ColorType;
use nvngx::FrameGenerationFeature;

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

    let _system = nvngx::System::new(
        None,
        env!("CARGO_PKG_VERSION"),
        &std::env::current_dir().unwrap(),
        &vk_mini_init.entry_fn,
        &vk_mini_init.instance,
        vk_mini_init.physical_device,
        vk_mini_init.device.handle(),
    )
    .unwrap();

    // Check support
    let capability_parameters =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("capability params");
    assert!(
        capability_parameters.supports_frame_generation().is_ok(),
        "Frame Generation not supported on this device"
    );

    let (width, height) = (1920, 1080);

    let create_params = nvngx::vk::FrameGenerationCreateParameters::new(
        width,
        height,
        vk::Format::R8G8B8A8_UNORM.as_raw() as u32,
        None,
        None,
        false,
    );

    // Create the feature
    let mut fg: nvngx_sys::Result<FrameGenerationFeature> =
        Err(nvngx::sys::Error::Other("Not initialized".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            fg = nvngx::vk::Feature::new_frame_generation(
                cb,
                capability_parameters,
                create_params,
            );
        })
        .unwrap();
    let mut fg = fg.expect("create Frame Generation feature");

    // Allocate GPU resources
    let mut allocator = vk_mini_init.get_allocator();

    let mut backbuffer_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        width,
        height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut depth_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        width,
        height,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut mv_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        width,
        height,
        vk::Format::R16G16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut output_interp_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        width,
        height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );
    let mut output_real_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        width,
        height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );
    let readback = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        (width * height * 4) as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );

    // Record evaluation
    vk_mini_init
        .record_and_submit(|cb, dev| {
            // Transition inputs
            for img in [&mut backbuffer_img, &mut depth_img, &mut mv_img] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::CLEAR,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }

            // Clear inputs with dummy data
            imgops::clear_color_image(dev, cb, backbuffer_img.image, [0.3, 0.5, 0.7, 1.0]);
            imgops::clear_color_image(dev, cb, depth_img.image, [1.0, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, mv_img.image, [0.0, 0.0, 0.0, 0.0]);

            // Transition for shader read
            for img in [&mut backbuffer_img, &mut depth_img, &mut mv_img] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                    vk::ImageLayout::GENERAL,
                );
            }
            for img in [&mut output_interp_img, &mut output_real_img] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::CLEAR,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }

            let subresource = imgops::default_subresource_range();

            let make_desc = |img: &allocations::ImageAllocation, format, writable| {
                nvngx::vk::VkImageResourceDescription {
                    image_view: img.view,
                    image: img.image,
                    subresource_range: subresource,
                    format,
                    width,
                    height,
                    mode: if writable {
                        nvngx::vk::VkResourceMode::Writable
                    } else {
                        nvngx::vk::VkResourceMode::Readable
                    },
                }
            };

            // Set up evaluation parameters
            let eval = fg.get_evaluation_parameters_mut();
            eval.set_backbuffer(make_desc(
                &backbuffer_img,
                vk::Format::R8G8B8A8_UNORM,
                false,
            ));
            eval.set_depth(make_desc(&depth_img, vk::Format::R32_SFLOAT, false));
            eval.set_motion_vectors(
                make_desc(&mv_img, vk::Format::R16G16_SFLOAT, false),
                None,
            );
            eval.set_output_interpolated_frame(make_desc(
                &output_interp_img,
                vk::Format::R8G8B8A8_UNORM,
                true,
            ));
            eval.set_output_real_frame(make_desc(
                &output_real_img,
                vk::Format::R8G8B8A8_UNORM,
                true,
            ));

            // Camera parameters (identity-like for demo)
            let identity = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            eval.set_camera_view_to_clip(&identity);
            eval.set_clip_to_camera_view(&identity);
            eval.set_clip_to_prev_clip(&identity);
            eval.set_prev_clip_to_clip(&identity);
            eval.set_jitter_offset(0.0, 0.0);
            eval.set_camera_position([0.0, 0.0, 0.0]);
            eval.set_camera_vectors([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
            eval.set_camera_near_far(0.1, 1000.0);
            eval.set_camera_fov(std::f32::consts::FRAC_PI_2);
            eval.set_camera_aspect_ratio(width as f32 / height as f32);
            eval.set_color_buffers_hdr(false);
            eval.set_depth_inverted(false);
            eval.set_camera_motion_included(true);
            eval.set_reset(true);

            fg.evaluate(cb).expect("Frame Generation evaluate");

            // Readback the interpolated frame
            output_interp_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                output_interp_img.image,
                readback.buffer,
                width,
                height,
            );
        })
        .unwrap();

    // Save output
    let mapped = readback.allocation.mapped_slice().expect("readback mapped");
    image::save_buffer_with_format(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/frame_generation/interpolated.png"
        ),
        mapped,
        width,
        height,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("save png");

    println!("Frame Generation interpolated frame saved.");

    // Cleanup
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, backbuffer_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, depth_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, mv_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, output_interp_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, output_real_img);
}
