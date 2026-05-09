//! Combined DLSS + DLSS-RR (ReSTIR) + DLSS-G pipeline demo.
//!
//! Three phases run in one binary:
//!
//!   Phase 1 — plain DLSS upscaling on the baboon test image.
//!   Phase 2 — DLSS-RR (with diffuse + specular hit-distance buffers,
//!             the ReSTIR-style input set) iterated over 32 frames of
//!             a slowly-orbiting analytic ray-traced scene.
//!   Phase 3 — DLSS-G interpolating between the last two DLSS-RR
//!             outputs.
//!
//! Each phase produces a saved PNG so the contributions of each
//! feature are visible side-by-side.
//!
//! Note: DLSS and DLSS-RR are normally alternatives in a real engine
//! (DLSS-RR replaces denoising + upscaling). Running both in one
//! binary here is purely demonstration.
//!
//! Outputs (in this directory):
//!   - dlss_baboon.png      — phase 1
//!   - rr_prev.png          — phase 2 (frame N-1 denoised)
//!   - rr_current.png       — phase 2 (frame N denoised)
//!   - interpolated.png     — phase 3 (DLSS-G between rr_prev and rr_current)

#[path = "../common/mod.rs"]
mod common;
use common::{allocations, imgops, vk_mini_init};

use ash::vk;
use image::ColorType;
use nvngx::{FrameGenerationFeature, RayReconstructionFeature, SuperSamplingFeature};

const RR_FRAMES: u32 = 32;
const ANGLE_DELTA: f32 = 0.01; // radians per frame; roughly 0.57°.

#[repr(C)]
#[derive(Copy, Clone)]
struct ResTirParams {
    canvas_size: [u32; 2],
    jitter: [f32; 2],
    frame_index: u32,
    current_angle: f32,
    prev_angle: f32,
    _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct DepthMvParams {
    canvas_size: [u32; 2],
    _pad0: [f32; 2],
    _frame_index: u32,
    current_angle: f32,
    prev_angle: f32,
    _pad1: u32,
}

fn halton(mut i: u32, base: u32) -> f32 {
    let mut f = 1.0;
    let mut r = 0.0;
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

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

    let mut allocator = vk_mini_init.get_allocator();
    let example_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/combined");

    // ===== Phase 1: plain DLSS on the baboon =====
    phase_dlss(&vk_mini_init, &system, &mut allocator, example_dir);

    // ===== Phase 2: DLSS-RR over RR_FRAMES of an orbiting scene =====
    let (mut rr_prev_snapshot, mut rr_current_snapshot, target_w, target_h) =
        phase_ray_reconstruction(&vk_mini_init, &system, &mut allocator, example_dir);

    // ===== Phase 3: DLSS-G between the two snapshots =====
    phase_frame_generation(
        &vk_mini_init,
        &system,
        &mut allocator,
        &mut rr_prev_snapshot,
        &mut rr_current_snapshot,
        target_w,
        target_h,
        example_dir,
    );

    allocations::destroy_image(&vk_mini_init.device, &mut allocator, rr_prev_snapshot);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, rr_current_snapshot);

    println!("Combined demo complete. PNGs written to {example_dir}.");
}

// ---------- Phase 1 -------------------------------------------------------

fn phase_dlss(
    vk_mini_init: &vk_mini_init::VkMiniInit,
    system: &nvngx::System,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    example_dir: &str,
) {
    let capability =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("DLSS capability");
    if let Err(e) = capability.supports_super_sampling() {
        eprintln!("[phase 1] DLSS not supported: {e}");
        return;
    }

    let (src_rgba, src_w, src_h) = allocations::load_png_rgba8(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/upsample/baboon.png"
    ));
    let (dst_w, dst_h) = (src_w * 2, src_h * 2);

    let create_params = nvngx::vk::SuperSamplingCreateParameters::new(
        src_w,
        src_h,
        dst_w,
        dst_h,
        Some(nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_Balanced),
        None,
    );
    let mut ss: nvngx_sys::Result<SuperSamplingFeature> =
        Err(nvngx::sys::Error::Other("Not initialised".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            ss = system.create_super_sampling_feature(cb, capability, create_params);
        })
        .unwrap();
    let mut ss = ss.expect("[phase 1] create DLSS feature");

    let mut staging = allocations::create_buffer(
        &vk_mini_init.device,
        allocator,
        src_rgba.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    staging
        .allocation
        .mapped_slice_mut()
        .expect("[phase 1] staging mapped")
        .copy_from_slice(src_rgba.as_raw());

    let mut color = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        src_w,
        src_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut depth = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        src_w,
        src_h,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut mv = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        src_w,
        src_h,
        vk::Format::R16G16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    );
    let mut output = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        dst_w,
        dst_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );
    let readback = allocations::create_buffer(
        &vk_mini_init.device,
        allocator,
        (dst_w * dst_h * 4) as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );

    vk_mini_init
        .record_and_submit(|cb, dev| {
            for img in [&mut color, &mut depth, &mut mv, &mut output] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::ALL_TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }
            imgops::copy_buffer_to_image(dev, cb, staging.buffer, color.image, src_w, src_h);
            imgops::clear_color_image(dev, cb, depth.image, [0.5, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, mv.image, [0.0, 0.0, 0.0, 0.0]);

            for img in [&mut color, &mut depth, &mut mv] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                    vk::ImageLayout::GENERAL,
                );
            }

            let mk_desc = |img: &allocations::ImageAllocation, w, h, format, writable| {
                nvngx::vk::VkImageResourceDescription {
                    image_view: img.view,
                    image: img.image,
                    subresource_range: imgops::default_subresource_range(),
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

            let eval = ss.get_evaluation_parameters_mut();
            eval.set_color_input(mk_desc(&color, src_w, src_h, vk::Format::R8G8B8A8_UNORM, false));
            eval.set_color_output(mk_desc(
                &output,
                dst_w,
                dst_h,
                vk::Format::R8G8B8A8_UNORM,
                true,
            ));
            eval.set_depth_buffer(mk_desc(&depth, src_w, src_h, vk::Format::R32_SFLOAT, false));
            eval.set_motions_vectors(
                mk_desc(&mv, src_w, src_h, vk::Format::R16G16_SFLOAT, false),
                None,
            );
            eval.set_jitter_offsets(0.0, 0.0);
            eval.set_reset(true);
            eval.set_rendering_dimensions([0, 0], [src_w, src_h]);

            ss.evaluate(cb).expect("[phase 1] DLSS evaluate");

            output.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::ALL_TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(dev, cb, output.image, readback.buffer, dst_w, dst_h);
        })
        .unwrap();

    let mapped = readback
        .allocation
        .mapped_slice()
        .expect("[phase 1] readback mapped");
    image::save_buffer_with_format(
        format!("{example_dir}/dlss_baboon.png"),
        mapped,
        dst_w,
        dst_h,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("[phase 1] save dlss_baboon.png");
    println!("[phase 1] DLSS upscale {src_w}×{src_h} → {dst_w}×{dst_h} saved.");

    allocations::destroy_buffer(&vk_mini_init.device, allocator, staging);
    allocations::destroy_buffer(&vk_mini_init.device, allocator, readback);
    allocations::destroy_image(&vk_mini_init.device, allocator, color);
    allocations::destroy_image(&vk_mini_init.device, allocator, depth);
    allocations::destroy_image(&vk_mini_init.device, allocator, mv);
    allocations::destroy_image(&vk_mini_init.device, allocator, output);
}

// ---------- Phase 2 -------------------------------------------------------

fn phase_ray_reconstruction(
    vk_mini_init: &vk_mini_init::VkMiniInit,
    system: &nvngx::System,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    example_dir: &str,
) -> (
    allocations::ImageAllocation,
    allocations::ImageAllocation,
    u32,
    u32,
) {
    let capability =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("RR capability");
    if let Err(e) = capability.supports_ray_reconstruction() {
        eprintln!("[phase 2] DLSS-RR not supported: {e}");
        std::process::exit(1);
    }

    let (target_w, target_h) = (1920u32, 1080u32);
    let optimal = nvngx::vk::SuperSamplingOptimalSettings::get_optimal_settings(
        &capability,
        target_w,
        target_h,
        nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_Balanced,
    )
    .expect("[phase 2] optimal settings");
    let (render_w, render_h) = (optimal.render_width, optimal.render_height);
    println!(
        "[phase 2] DLSS-RR (ReSTIR) {render_w}×{render_h} → {target_w}×{target_h}, {RR_FRAMES} frames."
    );

    let create_params = nvngx::vk::RayReconstructionCreateParameters::from(optimal).with_flags(
        nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_IsHDR
            | nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_AutoExposure
            | nvngx::sys::NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_MVLowRes,
    );
    let mut rr: nvngx_sys::Result<RayReconstructionFeature> =
        Err(nvngx::sys::Error::Other("Not initialised".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            rr = system.create_ray_reconstruction_feature(cb, capability, create_params);
        })
        .unwrap();
    let mut rr = rr.expect("[phase 2] create RR feature");

    // Render-res G-buffer.
    let storage_usage = vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::STORAGE;
    let mut color = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R16G16B16A16_SFLOAT,
        storage_usage,
    );
    let mut depth = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R32_SFLOAT,
        storage_usage,
    );
    let mut normals = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R16G16B16A16_SFLOAT,
        storage_usage,
    );
    let mut roughness = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R8_UNORM,
        storage_usage,
    );
    let mut diffuse_albedo = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R8G8B8A8_UNORM,
        storage_usage,
    );
    let mut specular_albedo = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R8G8B8A8_UNORM,
        storage_usage,
    );
    let mut mv = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R16G16_SFLOAT,
        storage_usage,
    );
    let mut diffuse_hit_dist = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R32_SFLOAT,
        storage_usage,
    );
    let mut specular_hit_dist = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        render_w,
        render_h,
        vk::Format::R32_SFLOAT,
        storage_usage,
    );
    // Target-res RR output. RGBA8 so it doubles as the DLSS-G
    // backbuffer with NativeBackbufferFormat = R8G8B8A8_UNORM.
    let mut rr_output = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );

    // Two snapshots that survive past phase 2 for DLSS-G to consume.
    let mut snapshot_prev = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::STORAGE,
    );
    let mut snapshot_current = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::STORAGE,
    );

    // Compile shader and build pipeline.
    let shader_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scene_restir.spv"));
    let (pipeline_layout, descriptor_pool, descriptor_set, compute_pipeline, shader_module, dsl) =
        build_compute_pipeline(
            &vk_mini_init.device,
            shader_bytes,
            &[
                (0, &color),
                (1, &depth),
                (2, &normals),
                (3, &roughness),
                (4, &diffuse_albedo),
                (5, &specular_albedo),
                (6, &mv),
                (7, &diffuse_hit_dist),
                (8, &specular_hit_dist),
            ],
            9,
            std::mem::size_of::<ResTirParams>() as u64,
        );
    let ubo_buffer_size = std::mem::size_of::<ResTirParams>() as u64;
    let mut ubo = allocations::create_buffer(
        &vk_mini_init.device,
        allocator,
        ubo_buffer_size,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    bind_ubo_to_descriptor_set(&vk_mini_init.device, descriptor_set, 9, ubo.buffer, ubo_buffer_size);

    let mk_desc = |img: &allocations::ImageAllocation, format, w, h, writable| {
        nvngx::vk::VkImageResourceDescription {
            image_view: img.view,
            image: img.image,
            subresource_range: imgops::default_subresource_range(),
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

    for frame_index in 0..RR_FRAMES {
        let jitter_x = halton(frame_index + 1, 2) - 0.5;
        let jitter_y = halton(frame_index + 1, 3) - 0.5;
        let current_angle = frame_index as f32 * ANGLE_DELTA;
        let prev_angle = (frame_index as f32 - 1.0) * ANGLE_DELTA;

        let params = ResTirParams {
            canvas_size: [render_w, render_h],
            jitter: [jitter_x, jitter_y],
            frame_index,
            current_angle,
            prev_angle,
            _pad: 0,
        };
        write_pod_to_ubo(&mut ubo, &params);

        let is_first = frame_index == 0;
        let is_prev = frame_index == RR_FRAMES - 2;
        let is_current = frame_index == RR_FRAMES - 1;

        vk_mini_init
            .record_and_submit(|cb, dev| {
                for img in [
                    &mut color,
                    &mut depth,
                    &mut normals,
                    &mut roughness,
                    &mut diffuse_albedo,
                    &mut specular_albedo,
                    &mut mv,
                    &mut diffuse_hit_dist,
                    &mut specular_hit_dist,
                ] {
                    img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                }
                if is_first {
                    rr_output.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::ALL_TRANSFER,
                        vk::AccessFlags2::TRANSFER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                    snapshot_prev.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::ALL_TRANSFER,
                        vk::AccessFlags2::TRANSFER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                    snapshot_current.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::ALL_TRANSFER,
                        vk::AccessFlags2::TRANSFER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                }

                unsafe {
                    dev.cmd_bind_pipeline(cb, vk::PipelineBindPoint::COMPUTE, compute_pipeline);
                    dev.cmd_bind_descriptor_sets(
                        cb,
                        vk::PipelineBindPoint::COMPUTE,
                        pipeline_layout,
                        0,
                        std::slice::from_ref(&descriptor_set),
                        &[],
                    );
                    dev.cmd_dispatch(cb, render_w.div_ceil(8), render_h.div_ceil(8), 1);
                }

                for img in [
                    &mut color,
                    &mut depth,
                    &mut normals,
                    &mut roughness,
                    &mut diffuse_albedo,
                    &mut specular_albedo,
                    &mut mv,
                    &mut diffuse_hit_dist,
                    &mut specular_hit_dist,
                ] {
                    img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                }

                let eval = rr.get_evaluation_parameters_mut();
                eval.set_color_input(mk_desc(
                    &color,
                    vk::Format::R16G16B16A16_SFLOAT,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_color_output(mk_desc(
                    &rr_output,
                    vk::Format::R8G8B8A8_UNORM,
                    target_w,
                    target_h,
                    true,
                ));
                eval.set_depth_buffer(mk_desc(
                    &depth,
                    vk::Format::R32_SFLOAT,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_motions_vectors(
                    mk_desc(&mv, vk::Format::R16G16_SFLOAT, render_w, render_h, false),
                    None,
                );
                eval.set_normals(mk_desc(
                    &normals,
                    vk::Format::R16G16B16A16_SFLOAT,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_roughness(mk_desc(
                    &roughness,
                    vk::Format::R8_UNORM,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_diffuse_albedo(mk_desc(
                    &diffuse_albedo,
                    vk::Format::R8G8B8A8_UNORM,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_specular_albedo(mk_desc(
                    &specular_albedo,
                    vk::Format::R8G8B8A8_UNORM,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_diffuse_hit_distance(mk_desc(
                    &diffuse_hit_dist,
                    vk::Format::R32_SFLOAT,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_specular_hit_distance(mk_desc(
                    &specular_hit_dist,
                    vk::Format::R32_SFLOAT,
                    render_w,
                    render_h,
                    false,
                ));
                eval.set_jitter_offsets(jitter_x, jitter_y);
                eval.set_reset(is_first);
                eval.set_rendering_dimensions([0, 0], [render_w, render_h]);

                rr.evaluate(cb).expect("[phase 2] RR evaluate");

                if is_prev || is_current {
                    rr_output.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::ALL_TRANSFER,
                        vk::AccessFlags2::TRANSFER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                    let dst = if is_prev {
                        &mut snapshot_prev
                    } else {
                        &mut snapshot_current
                    };
                    dst.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::ALL_TRANSFER,
                        vk::AccessFlags2::TRANSFER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                    let region = vk::ImageCopy::default()
                        .src_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1),
                        )
                        .dst_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1),
                        )
                        .extent(vk::Extent3D {
                            width: target_w,
                            height: target_h,
                            depth: 1,
                        });
                    unsafe {
                        dev.cmd_copy_image(
                            cb,
                            rr_output.image,
                            vk::ImageLayout::GENERAL,
                            dst.image,
                            vk::ImageLayout::GENERAL,
                            std::slice::from_ref(&region),
                        );
                    }
                }
            })
            .unwrap();
    }

    // Read back snapshots and save them.
    for (snap, name) in [
        (&mut snapshot_prev, "rr_prev.png"),
        (&mut snapshot_current, "rr_current.png"),
    ] {
        let readback = allocations::create_buffer(
            &vk_mini_init.device,
            allocator,
            (target_w * target_h * 4) as u64,
            vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuToCpu,
        );
        vk_mini_init
            .record_and_submit(|cb, dev| {
                snap.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::ALL_TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                    vk::ImageLayout::GENERAL,
                );
                imgops::copy_image_to_buffer(
                    dev,
                    cb,
                    snap.image,
                    readback.buffer,
                    target_w,
                    target_h,
                );
            })
            .unwrap();
        let mapped = readback
            .allocation
            .mapped_slice()
            .expect("[phase 2] readback mapped");
        image::save_buffer_with_format(
            format!("{example_dir}/{name}"),
            mapped,
            target_w,
            target_h,
            ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap_or_else(|_| panic!("[phase 2] save {name}"));
        allocations::destroy_buffer(&vk_mini_init.device, allocator, readback);
    }
    println!("[phase 2] rr_prev.png and rr_current.png saved.");

    // Tear down phase 2's resources except the two snapshots which
    // survive into phase 3.
    unsafe {
        vk_mini_init.device.destroy_pipeline(compute_pipeline, None);
        vk_mini_init
            .device
            .destroy_pipeline_layout(pipeline_layout, None);
        vk_mini_init
            .device
            .destroy_descriptor_pool(descriptor_pool, None);
        vk_mini_init
            .device
            .destroy_descriptor_set_layout(dsl, None);
        vk_mini_init
            .device
            .destroy_shader_module(shader_module, None);
    }
    allocations::destroy_buffer(&vk_mini_init.device, allocator, ubo);
    allocations::destroy_image(&vk_mini_init.device, allocator, color);
    allocations::destroy_image(&vk_mini_init.device, allocator, depth);
    allocations::destroy_image(&vk_mini_init.device, allocator, normals);
    allocations::destroy_image(&vk_mini_init.device, allocator, roughness);
    allocations::destroy_image(&vk_mini_init.device, allocator, diffuse_albedo);
    allocations::destroy_image(&vk_mini_init.device, allocator, specular_albedo);
    allocations::destroy_image(&vk_mini_init.device, allocator, mv);
    allocations::destroy_image(&vk_mini_init.device, allocator, diffuse_hit_dist);
    allocations::destroy_image(&vk_mini_init.device, allocator, specular_hit_dist);
    allocations::destroy_image(&vk_mini_init.device, allocator, rr_output);

    (snapshot_prev, snapshot_current, target_w, target_h)
}

// ---------- Phase 3 -------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn phase_frame_generation(
    vk_mini_init: &vk_mini_init::VkMiniInit,
    system: &nvngx::System,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    snapshot_prev: &mut allocations::ImageAllocation,
    snapshot_current: &mut allocations::ImageAllocation,
    target_w: u32,
    target_h: u32,
    example_dir: &str,
) {
    let capability =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("FG capability");
    if let Err(e) = capability.supports_frame_generation() {
        eprintln!("[phase 3] DLSS-G not supported: {e}");
        return;
    }

    let create_params = nvngx::vk::FrameGenerationCreateParameters::new(
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM.as_raw() as u32,
        None,
        None,
        false,
    );
    let mut fg: nvngx_sys::Result<FrameGenerationFeature> =
        Err(nvngx::sys::Error::Other("Not initialised".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            fg = system.create_frame_generation_feature(cb, capability, create_params);
        })
        .unwrap();
    let mut fg = fg.expect("[phase 3] create FG feature");
    println!(
        "[phase 3] DLSS-G initialised. multiFrameCountMax = {}.",
        fg.multi_frame_count_max()
    );

    // Target-resolution depth and motion-vector images.
    let storage_usage = vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::STORAGE;
    let mut depth = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R32_SFLOAT,
        storage_usage,
    );
    let mut mv = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R16G16_SFLOAT,
        storage_usage,
    );
    let mut output_interp = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );
    let mut output_real = allocations::create_image_optimal(
        &vk_mini_init.device,
        allocator,
        target_w,
        target_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE,
    );

    let shader_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/depth_mv.spv"));
    let (pipeline_layout, descriptor_pool, descriptor_set, compute_pipeline, shader_module, dsl) =
        build_compute_pipeline(
            &vk_mini_init.device,
            shader_bytes,
            &[(0, &depth), (1, &mv)],
            2,
            std::mem::size_of::<DepthMvParams>() as u64,
        );
    let ubo_size = std::mem::size_of::<DepthMvParams>() as u64;
    let mut ubo = allocations::create_buffer(
        &vk_mini_init.device,
        allocator,
        ubo_size,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    bind_ubo_to_descriptor_set(&vk_mini_init.device, descriptor_set, 2, ubo.buffer, ubo_size);

    let mk_desc = |img: &allocations::ImageAllocation, format, writable| {
        nvngx::vk::VkImageResourceDescription {
            image_view: img.view,
            image: img.image,
            subresource_range: imgops::default_subresource_range(),
            format,
            width: target_w,
            height: target_h,
            mode: if writable {
                nvngx::vk::VkResourceMode::Writable
            } else {
                nvngx::vk::VkResourceMode::Readable
            },
        }
    };

    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    // We re-issue the depth+MV dispatch for the matching camera angle
    // before each FG evaluate. The first evaluate (for the "previous"
    // frame) primes the snippet's temporal cache with reset=true; the
    // second evaluate produces the interpolated frame.
    for (which, snapshot) in [
        ("prev", &mut *snapshot_prev),
        ("current", &mut *snapshot_current),
    ] {
        let is_prev = which == "prev";
        let frame_index = if is_prev {
            RR_FRAMES - 2
        } else {
            RR_FRAMES - 1
        };
        let current_angle = frame_index as f32 * ANGLE_DELTA;
        let prev_angle = (frame_index as f32 - 1.0) * ANGLE_DELTA;

        let params = DepthMvParams {
            canvas_size: [target_w, target_h],
            _pad0: [0.0; 2],
            _frame_index: frame_index,
            current_angle,
            prev_angle,
            _pad1: 0,
        };
        write_pod_to_ubo(&mut ubo, &params);

        vk_mini_init
            .record_and_submit(|cb, dev| {
                for img in [&mut depth, &mut mv] {
                    img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_WRITE,
                        vk::ImageLayout::GENERAL,
                    );
                }
                if is_prev {
                    for img in [&mut output_interp, &mut output_real] {
                        img.image_barrier(
                            dev,
                            cb,
                            vk::PipelineStageFlags2::ALL_TRANSFER,
                            vk::AccessFlags2::TRANSFER_WRITE,
                            vk::ImageLayout::GENERAL,
                        );
                    }
                }

                unsafe {
                    dev.cmd_bind_pipeline(cb, vk::PipelineBindPoint::COMPUTE, compute_pipeline);
                    dev.cmd_bind_descriptor_sets(
                        cb,
                        vk::PipelineBindPoint::COMPUTE,
                        pipeline_layout,
                        0,
                        std::slice::from_ref(&descriptor_set),
                        &[],
                    );
                    dev.cmd_dispatch(cb, target_w.div_ceil(8), target_h.div_ceil(8), 1);
                }

                for img in [&mut depth, &mut mv] {
                    img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                }
                snapshot.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                    vk::ImageLayout::GENERAL,
                );

                let eval = fg.get_evaluation_parameters_mut();
                eval.set_backbuffer(mk_desc(snapshot, vk::Format::R8G8B8A8_UNORM, false));
                eval.set_depth(mk_desc(&depth, vk::Format::R32_SFLOAT, false));
                eval.set_motion_vectors(
                    mk_desc(&mv, vk::Format::R16G16_SFLOAT, false),
                    None,
                );
                eval.set_output_interpolated_frame(mk_desc(
                    &output_interp,
                    vk::Format::R8G8B8A8_UNORM,
                    true,
                ));
                eval.set_output_real_frame(mk_desc(
                    &output_real,
                    vk::Format::R8G8B8A8_UNORM,
                    true,
                ));
                eval.set_camera_view_to_clip(&identity);
                eval.set_clip_to_camera_view(&identity);
                eval.set_clip_to_prev_clip(&identity);
                eval.set_prev_clip_to_clip(&identity);
                eval.set_jitter_offset(0.0, 0.0);
                eval.set_camera_position([0.0, 0.0, 0.0]);
                eval.set_camera_vectors([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
                eval.set_camera_near_far(0.1, 1000.0);
                eval.set_camera_fov(std::f32::consts::FRAC_PI_2);
                eval.set_camera_aspect_ratio(target_w as f32 / target_h as f32);
                eval.set_color_buffers_hdr(false);
                eval.set_depth_inverted(false);
                eval.set_camera_motion_included(true);
                eval.set_multi_frame(1, 1);
                eval.set_reset(is_prev);

                fg.evaluate(cb).expect("[phase 3] FG evaluate");

                // Mark snippet-touched images as fully synchronised
                // so subsequent barriers source from a sane state.
                for img in [
                    &mut depth,
                    &mut mv,
                    &mut output_interp,
                    &mut output_real,
                    snapshot,
                ] {
                    img.current_stage = vk::PipelineStageFlags2::ALL_COMMANDS;
                    img.current_access =
                        vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE;
                    img.current_layout = vk::ImageLayout::GENERAL;
                }
            })
            .unwrap();
    }

    // Read back the interpolated frame.
    let readback = allocations::create_buffer(
        &vk_mini_init.device,
        allocator,
        (target_w * target_h * 4) as u64,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );
    vk_mini_init
        .record_and_submit(|cb, dev| {
            output_interp.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::ALL_TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                output_interp.image,
                readback.buffer,
                target_w,
                target_h,
            );
        })
        .unwrap();
    let mapped = readback
        .allocation
        .mapped_slice()
        .expect("[phase 3] readback mapped");
    image::save_buffer_with_format(
        format!("{example_dir}/interpolated.png"),
        mapped,
        target_w,
        target_h,
        ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("[phase 3] save interpolated.png");
    println!("[phase 3] interpolated.png saved.");

    unsafe {
        vk_mini_init.device.destroy_pipeline(compute_pipeline, None);
        vk_mini_init
            .device
            .destroy_pipeline_layout(pipeline_layout, None);
        vk_mini_init
            .device
            .destroy_descriptor_pool(descriptor_pool, None);
        vk_mini_init
            .device
            .destroy_descriptor_set_layout(dsl, None);
        vk_mini_init
            .device
            .destroy_shader_module(shader_module, None);
    }
    allocations::destroy_buffer(&vk_mini_init.device, allocator, ubo);
    allocations::destroy_buffer(&vk_mini_init.device, allocator, readback);
    allocations::destroy_image(&vk_mini_init.device, allocator, depth);
    allocations::destroy_image(&vk_mini_init.device, allocator, mv);
    allocations::destroy_image(&vk_mini_init.device, allocator, output_interp);
    allocations::destroy_image(&vk_mini_init.device, allocator, output_real);
}

// ---------- Compute-pipeline helpers --------------------------------------

#[allow(clippy::type_complexity)]
fn build_compute_pipeline(
    device: &ash::Device,
    spirv_bytes: &[u8],
    image_bindings: &[(u32, &allocations::ImageAllocation)],
    ubo_binding: u32,
    _ubo_size: u64,
) -> (
    vk::PipelineLayout,
    vk::DescriptorPool,
    vk::DescriptorSet,
    vk::Pipeline,
    vk::ShaderModule,
    vk::DescriptorSetLayout,
) {
    let words: Vec<u32> = spirv_bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let shader_ci = vk::ShaderModuleCreateInfo::default().code(&words);
    let shader_module = unsafe { device.create_shader_module(&shader_ci, None) }
        .expect("create shader module");

    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = image_bindings
        .iter()
        .map(|(b, _)| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(*b)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        })
        .collect();
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(ubo_binding)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    );
    let dsl_ci = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let dsl = unsafe { device.create_descriptor_set_layout(&dsl_ci, None) }
        .expect("create dsl");

    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(image_bindings.len() as u32),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1),
    ];
    let pool_ci = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&pool_sizes);
    let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_ci, None) }
        .expect("create pool");

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&dsl));
    let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info) }
        .expect("alloc set")[0];

    let image_infos: Vec<vk::DescriptorImageInfo> = image_bindings
        .iter()
        .map(|(_, img)| {
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(img.view)
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = image_bindings
        .iter()
        .enumerate()
        .map(|(i, (b, _))| {
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(*b)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&image_infos[i]))
        })
        .collect();
    unsafe { device.update_descriptor_sets(&writes, &[]) };

    let pl_ci = vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&dsl));
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pl_ci, None) }.expect("create pipeline layout");

    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(shader_module)
        .name(c"main");
    let pipeline_ci = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(pipeline_layout);
    let compute_pipeline = unsafe {
        device.create_compute_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_ci),
            None,
        )
    }
    .expect("create compute pipeline")[0];

    (
        pipeline_layout,
        descriptor_pool,
        descriptor_set,
        compute_pipeline,
        shader_module,
        dsl,
    )
}

fn bind_ubo_to_descriptor_set(
    device: &ash::Device,
    set: vk::DescriptorSet,
    binding: u32,
    buffer: vk::Buffer,
    range: u64,
) {
    let info = vk::DescriptorBufferInfo::default()
        .buffer(buffer)
        .offset(0)
        .range(range);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(binding)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(std::slice::from_ref(&info));
    unsafe { device.update_descriptor_sets(std::slice::from_ref(&write), &[]) };
}

fn write_pod_to_ubo<T: Copy>(ubo: &mut allocations::BufferAllocation, value: &T) {
    let bytes = unsafe {
        std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
    };
    let mapped = ubo.allocation.mapped_slice_mut().expect("ubo mapped");
    mapped[..bytes.len()].copy_from_slice(bytes);
}
