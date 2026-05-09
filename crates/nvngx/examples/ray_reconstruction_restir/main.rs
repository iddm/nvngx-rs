//! Ray Reconstruction with ReSTIR - demonstrates hit distance resources.

#[path = "../common/mod.rs"]
mod common;
use common::{allocations, imgops, vk_mini_init};

use ash::vk;
use image::ColorType;
use nvngx::RayReconstructionFeature;

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
        &std::env::current_dir().unwrap(),
        &vk_mini_init.entry_fn,
        &vk_mini_init.instance,
        vk_mini_init.physical_device,
        vk_mini_init.device.handle(),
    )
    .unwrap();

    let capability_parameters =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("capability params");
    if let Err(e) = capability_parameters.supports_ray_reconstruction() {
        eprintln!("Ray Reconstruction not supported on this device: {e}");
        std::process::exit(1);
    }

    let (dst_width, dst_height) = (1920, 1080);

    let optimal_settings = nvngx::vk::SuperSamplingOptimalSettings::get_optimal_settings(
        &capability_parameters,
        dst_width,
        dst_height,
        nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_Balanced,
    )
    .expect("optimal settings");

    // DLSS-RR requires the host to advertise an HDR color buffer
    // (the snippet refuses CreateFeature with `Error: HDR Color
    // required` otherwise). Our color image below is RGBA16F, so
    // the claim is honest. AutoExposure tells the snippet to derive
    // exposure internally (no exposure texture supplied);
    // MVLowRes signals that motion vectors are at the render
    // resolution, not the upscaled target resolution.
    let create_params =
        nvngx::vk::RayReconstructionCreateParameters::from(optimal_settings).with_flags(
            nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_IsHDR
                | nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_AutoExposure
                | nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_MVLowRes,
        );

    let mut rr: nvngx_sys::Result<RayReconstructionFeature> =
        Err(nvngx::sys::Error::Other("Not initialized".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            rr = system.create_ray_reconstruction_feature(cb, capability_parameters, create_params);
        })
        .unwrap();
    let mut rr = rr.expect("create Ray Reconstruction feature");

    let mut allocator = vk_mini_init.get_allocator();

    let render_width = optimal_settings.render_width;
    let render_height = optimal_settings.render_height;

    // Standard inputs. Color is RGBA16F so it can carry HDR values
    // — DLSS-RR's `IsHDR` create-flag claim must be backed by an
    // actual HDR-capable format here.
    let mut color_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut mv_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R16G16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut depth_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut diffuse_albedo_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut specular_albedo_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut normals_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut roughness_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );

    // ReSTIR-specific: hit distance resources
    let mut diffuse_hit_dist_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut specular_hit_dist_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
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

    vk_mini_init
        .record_and_submit(|cb, dev| {
            // Transition all inputs
            for img in [
                &mut color_img,
                &mut mv_img,
                &mut depth_img,
                &mut diffuse_albedo_img,
                &mut specular_albedo_img,
                &mut normals_img,
                &mut roughness_img,
                &mut diffuse_hit_dist_img,
                &mut specular_hit_dist_img,
            ] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::CLEAR,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }

            // Clear inputs
            imgops::clear_color_image(dev, cb, color_img.image, [0.5, 0.5, 0.5, 1.0]);
            imgops::clear_color_image(dev, cb, mv_img.image, [0.0, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, depth_img.image, [1.0, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, diffuse_albedo_img.image, [0.5, 0.5, 0.5, 1.0]);
            imgops::clear_color_image(dev, cb, specular_albedo_img.image, [0.04, 0.04, 0.04, 1.0]);
            imgops::clear_color_image(dev, cb, normals_img.image, [0.0, 0.0, 1.0, 0.0]);
            imgops::clear_color_image(dev, cb, roughness_img.image, [0.5, 0.0, 0.0, 0.0]);
            // ReSTIR hit distances (simulated)
            imgops::clear_color_image(dev, cb, diffuse_hit_dist_img.image, [10.0, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(
                dev,
                cb,
                specular_hit_dist_img.image,
                [5.0, 0.0, 0.0, 0.0],
            );

            // Transition for shader read
            for img in [
                &mut color_img,
                &mut mv_img,
                &mut depth_img,
                &mut diffuse_albedo_img,
                &mut specular_albedo_img,
                &mut normals_img,
                &mut roughness_img,
                &mut diffuse_hit_dist_img,
                &mut specular_hit_dist_img,
            ] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                    vk::ImageLayout::GENERAL,
                );
            }
            out_img.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );

            let subresource = imgops::default_subresource_range();

            let make_desc =
                |img: &allocations::ImageAllocation, format, w, h, writable| {
                    nvngx::vk::VkImageResourceDescription {
                        image_view: img.view,
                        image: img.image,
                        subresource_range: subresource,
                        format,
                        width: w,
                        height: h,
                        mode: if writable {
                            nvngx::vk::VkResourceMode::Writable
                        } else {
                            nvngx::vk::VkResourceMode::Readable
                        },
                    }
                };

            let eval = rr.get_evaluation_parameters_mut();
            eval.set_color_input(make_desc(
                &color_img,
                vk::Format::R16G16B16A16_SFLOAT,
                render_width,
                render_height,
                false,
            ));
            eval.set_color_output(make_desc(
                &out_img,
                vk::Format::R8G8B8A8_UNORM,
                dst_width,
                dst_height,
                true,
            ));
            eval.set_motions_vectors(
                make_desc(
                    &mv_img,
                    vk::Format::R16G16_SFLOAT,
                    render_width,
                    render_height,
                    false,
                ),
                None,
            );
            eval.set_depth_buffer(make_desc(
                &depth_img,
                vk::Format::R32_SFLOAT,
                render_width,
                render_height,
                false,
            ));
            eval.set_diffuse_albedo(make_desc(
                &diffuse_albedo_img,
                vk::Format::R8G8B8A8_UNORM,
                render_width,
                render_height,
                false,
            ));
            eval.set_specular_albedo(make_desc(
                &specular_albedo_img,
                vk::Format::R8G8B8A8_UNORM,
                render_width,
                render_height,
                false,
            ));
            eval.set_normals(make_desc(
                &normals_img,
                vk::Format::R16G16B16A16_SFLOAT,
                render_width,
                render_height,
                false,
            ));
            eval.set_roughness(make_desc(
                &roughness_img,
                vk::Format::R8_UNORM,
                render_width,
                render_height,
                false,
            ));
            // ReSTIR-specific: hit distances
            eval.set_diffuse_hit_distance(make_desc(
                &diffuse_hit_dist_img,
                vk::Format::R32_SFLOAT,
                render_width,
                render_height,
                false,
            ));
            eval.set_specular_hit_distance(make_desc(
                &specular_hit_dist_img,
                vk::Format::R32_SFLOAT,
                render_width,
                render_height,
                false,
            ));
            eval.set_jitter_offsets(0.0, 0.0);
            eval.set_reset(true);
            eval.set_rendering_dimensions([0, 0], [render_width, render_height]);

            rr.evaluate(cb)
                .expect("Ray Reconstruction + ReSTIR evaluate");

            // Readback
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

    let mapped = readback.allocation.mapped_slice().expect("readback mapped");
    image::save_buffer_with_format(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/ray_reconstruction_restir/output.png"
        ),
        mapped,
        dst_width,
        dst_height,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("save png");

    println!("Ray Reconstruction + ReSTIR output saved.");

    // Cleanup
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, color_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, mv_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, depth_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, diffuse_albedo_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, specular_albedo_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, normals_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, roughness_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, diffuse_hit_dist_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, specular_hit_dist_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, out_img);
}
