//! DLSS-RR (Ray Reconstruction) demo.
//!
//! Renders a procedural ray-traced scene (a checker plane + three
//! spheres) one sample per pixel. With one sample, the shaded color
//! is heavily dominated by Monte Carlo noise — that is the raw input
//! DLSS-RR is designed to denoise. The full G-buffer (depth, normals,
//! roughness, diffuse/specular albedo, motion vectors) is written
//! alongside the color so DLSS-RR can use spatial guidance.
//!
//! The example then iterates DLSS-RR over `FRAMES` frames with sub-
//! pixel Halton(2, 3) jitter, accumulating temporally. The first
//! frame's noisy shader output and the last frame's DLSS-RR output
//! are saved as `noisy_input.png` and `denoised.png` next to this
//! source. The before/after comparison is the visible "it works"
//! signal.

#[path = "../common/mod.rs"]
mod common;
use common::{allocations, imgops, vk_mini_init};

use ash::vk;
use image::ColorType;
use nvngx::RayReconstructionFeature;

const FRAMES: u32 = 32;

#[repr(C)]
#[derive(Copy, Clone)]
struct ShaderParams {
    canvas_size: [u32; 2],
    jitter: [f32; 2],
    frame_index: u32,
    _pad: [u32; 3],
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

    let capability_parameters =
        nvngx::vk::FeatureParameters::get_capability_parameters().expect("capability params");
    if let Err(e) = capability_parameters.supports_ray_reconstruction() {
        eprintln!("Ray Reconstruction not supported on this device: {e}");
        std::process::exit(1);
    }

    let (target_width, target_height) = (1920u32, 1080u32);
    let optimal = nvngx::vk::SuperSamplingOptimalSettings::get_optimal_settings(
        &capability_parameters,
        target_width,
        target_height,
        nvngx::sys::NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_MaxQuality,
    )
    .expect("optimal settings");
    let (render_width, render_height) = (optimal.render_width, optimal.render_height);
    println!(
        "DLSS-RR optimal settings: render {render_width}×{render_height} → \
         target {target_width}×{target_height} (Balanced)."
    );

    // DLSS-RR rejects `CreateFeature` if the create flags don't
    // describe the host's G-buffer:
    //   - IsHDR is required (the snippet logs `Error: HDR Color
    //     required` otherwise); our color buffer is RGBA16F.
    //   - AutoExposure tells the snippet to derive exposure
    //     internally — we don't supply an exposure texture.
    //   - MVLowRes signals motion vectors are at render resolution
    //     rather than upscaled target resolution.
    // Jitter is applied to ray generation inside the shader and
    // reported via `set_jitter_offsets`, not encoded into the MV
    // buffer, so MVJittered is left off.
    let create_params = nvngx::vk::RayReconstructionCreateParameters::from(optimal).with_flags(
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

    // ===== G-buffer images, one per shader output =====
    let color_format = vk::Format::R16G16B16A16_SFLOAT;
    let depth_format = vk::Format::R32_SFLOAT;
    let normals_format = vk::Format::R16G16B16A16_SFLOAT;
    let roughness_format = vk::Format::R8_UNORM;
    let albedo_format = vk::Format::R8G8B8A8_UNORM;
    let mv_format = vk::Format::R16G16_SFLOAT;
    let output_format = vk::Format::R8G8B8A8_UNORM;

    let storage_usage = vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::STORAGE;

    let mut color_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        color_format,
        storage_usage,
    );
    let mut depth_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        depth_format,
        storage_usage,
    );
    let mut normals_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        normals_format,
        storage_usage,
    );
    let mut roughness_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        roughness_format,
        storage_usage,
    );
    let mut diffuse_albedo_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        albedo_format,
        storage_usage,
    );
    let mut specular_albedo_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        albedo_format,
        storage_usage,
    );
    let mut mv_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        render_width,
        render_height,
        mv_format,
        storage_usage,
    );
    let mut out_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        target_width,
        target_height,
        output_format,
        vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE,
    );

    // ===== Compute pipeline for the scene shader =====
    let shader_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scene.spv"));
    let shader_words: Vec<u32> = shader_bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let shader_module_ci = vk::ShaderModuleCreateInfo::default().code(&shader_words);
    let shader_module = unsafe {
        vk_mini_init
            .device
            .create_shader_module(&shader_module_ci, None)
    }
    .expect("create shader module");

    // 8 bindings: 7 storage images + 1 uniform buffer.
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..7u32)
        .map(|b| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(b)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        })
        .chain(std::iter::once(
            vk::DescriptorSetLayoutBinding::default()
                .binding(7)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ))
        .collect();
    let dsl_ci = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let dsl = unsafe {
        vk_mini_init
            .device
            .create_descriptor_set_layout(&dsl_ci, None)
    }
    .expect("create dsl");

    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(7),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1),
    ];
    let pool_ci = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&pool_sizes);
    let descriptor_pool = unsafe {
        vk_mini_init
            .device
            .create_descriptor_pool(&pool_ci, None)
    }
    .expect("create descriptor pool");

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&dsl));
    let descriptor_set = unsafe {
        vk_mini_init
            .device
            .allocate_descriptor_sets(&alloc_info)
    }
    .expect("alloc descriptor set")[0];

    let pl_layout_ci =
        vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&dsl));
    let pipeline_layout = unsafe {
        vk_mini_init
            .device
            .create_pipeline_layout(&pl_layout_ci, None)
    }
    .expect("create pipeline layout");

    let stage_ci = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(shader_module)
        .name(c"main");
    let pipeline_ci = vk::ComputePipelineCreateInfo::default()
        .stage(stage_ci)
        .layout(pipeline_layout);
    let compute_pipeline = unsafe {
        vk_mini_init.device.create_compute_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_ci),
            None,
        )
    }
    .expect("create compute pipeline")[0];

    // ===== UBO buffer (host-visible, persistently mapped) =====
    let ubo_size = std::mem::size_of::<ShaderParams>() as u64;
    let mut ubo = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        ubo_size,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );

    // Wire the descriptor set.
    let storage_image_infos: Vec<vk::DescriptorImageInfo> = [
        &color_img,
        &depth_img,
        &normals_img,
        &roughness_img,
        &diffuse_albedo_img,
        &specular_albedo_img,
        &mv_img,
    ]
    .iter()
    .map(|img| {
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(img.view)
    })
    .collect();
    let ubo_info = vk::DescriptorBufferInfo::default()
        .buffer(ubo.buffer)
        .offset(0)
        .range(ubo_size);

    let mut writes: Vec<vk::WriteDescriptorSet> = (0..7u32)
        .map(|b| {
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(b)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&storage_image_infos[b as usize]))
        })
        .collect();
    writes.push(
        vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(7)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&ubo_info)),
    );
    unsafe {
        vk_mini_init.device.update_descriptor_sets(&writes, &[]);
    }

    // ===== Readback buffers =====
    let render_byte_len = (render_width * render_height * 8) as u64; // RGBA16F = 8 bytes
    let target_byte_len = (target_width * target_height * 4) as u64;
    let readback_noisy = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        render_byte_len,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );
    let readback_denoised = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        target_byte_len,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );

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

    // ===== Per-frame loop =====
    println!(
        "DLSS-RR demo: rendering {render_width}×{render_height} → {target_width}×{target_height}, \
         {FRAMES} frames with Halton(2,3) jitter."
    );
    for frame_index in 0..FRAMES {
        // Halton jitter, centered at 0, range ±0.5 px.
        let jitter_x = halton(frame_index + 1, 2) - 0.5;
        let jitter_y = halton(frame_index + 1, 3) - 0.5;

        // Update UBO contents.
        let params = ShaderParams {
            canvas_size: [render_width, render_height],
            jitter: [jitter_x, jitter_y],
            frame_index,
            _pad: [0; 3],
        };
        let bytes: [u8; std::mem::size_of::<ShaderParams>()] = unsafe {
            std::mem::transmute::<ShaderParams, [u8; std::mem::size_of::<ShaderParams>()]>(params)
        };
        // Allocator may have rounded the buffer up for UBO alignment;
        // only write the bytes we care about.
        let mapped = ubo.allocation.mapped_slice_mut().expect("ubo mapped");
        mapped[..bytes.len()].copy_from_slice(&bytes);

        let is_first = frame_index == 0;
        let is_last = frame_index == FRAMES - 1;

        vk_mini_init
            .record_and_submit(|cb, dev| {
                // Move every G-buffer image to GENERAL for compute writes.
                for img in [
                    &mut color_img,
                    &mut depth_img,
                    &mut normals_img,
                    &mut roughness_img,
                    &mut diffuse_albedo_img,
                    &mut specular_albedo_img,
                    &mut mv_img,
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
                    out_img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::CLEAR,
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
                    dev.cmd_dispatch(
                        cb,
                        render_width.div_ceil(8),
                        render_height.div_ceil(8),
                        1,
                    );
                }

                // Compute writes complete → DLSS-RR reads from same images.
                for img in [
                    &mut color_img,
                    &mut depth_img,
                    &mut normals_img,
                    &mut roughness_img,
                    &mut diffuse_albedo_img,
                    &mut specular_albedo_img,
                    &mut mv_img,
                ] {
                    img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                }

                // Snapshot the first frame's noisy color before DLSS-RR
                // touches anything, so the user can see what the input
                // looks like.
                if is_first {
                    color_img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::TRANSFER,
                        vk::AccessFlags2::TRANSFER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                    imgops::copy_image_to_buffer(
                        dev,
                        cb,
                        color_img.image,
                        readback_noisy.buffer,
                        render_width,
                        render_height,
                    );
                    color_img.image_barrier(
                        dev,
                        cb,
                        vk::PipelineStageFlags2::COMPUTE_SHADER,
                        vk::AccessFlags2::SHADER_READ,
                        vk::ImageLayout::GENERAL,
                    );
                }

                // Wire DLSS-RR.
                let eval = rr.get_evaluation_parameters_mut();
                eval.set_color_input(mk_desc(
                    &color_img,
                    color_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_color_output(mk_desc(
                    &out_img,
                    output_format,
                    target_width,
                    target_height,
                    true,
                ));
                eval.set_motions_vectors(
                    mk_desc(&mv_img, mv_format, render_width, render_height, false),
                    None,
                );
                eval.set_depth_buffer(mk_desc(
                    &depth_img,
                    depth_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_diffuse_albedo(mk_desc(
                    &diffuse_albedo_img,
                    albedo_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_specular_albedo(mk_desc(
                    &specular_albedo_img,
                    albedo_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_normals(mk_desc(
                    &normals_img,
                    normals_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_roughness(mk_desc(
                    &roughness_img,
                    roughness_format,
                    render_width,
                    render_height,
                    false,
                ));
                eval.set_jitter_offsets(jitter_x, jitter_y);
                eval.set_reset(is_first);
                eval.set_rendering_dimensions([0, 0], [render_width, render_height]);

                rr.evaluate(cb).expect("DLSS-RR evaluate");

                if is_last {
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
                        readback_denoised.buffer,
                        target_width,
                        target_height,
                    );
                }
            })
            .unwrap();
    }

    // ===== Save outputs =====
    let example_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/ray_reconstruction");

    // The noisy frame is RGBA16F. Convert to 8-bit by tonemapping
    // (Reinhard) and clamping. This keeps the saved file useful for a
    // "see how noisy 1 spp is" visual.
    {
        let raw: &[u8] = readback_noisy.allocation.mapped_slice().expect("readback_noisy mapped");
        let pixel_count = (render_width * render_height) as usize;
        let mut rgba8 = Vec::with_capacity(pixel_count * 4);
        let read_f16 = |b: [u8; 2]| half::f16::from_le_bytes(b).to_f32();
        for i in 0..pixel_count {
            let off = i * 8;
            let r = read_f16([raw[off], raw[off + 1]]);
            let g = read_f16([raw[off + 2], raw[off + 3]]);
            let b = read_f16([raw[off + 4], raw[off + 5]]);
            let a = read_f16([raw[off + 6], raw[off + 7]]);
            // Reinhard tonemap to keep the noise spike visible without crushing.
            let tone = |x: f32| (x / (1.0 + x)).clamp(0.0, 1.0).powf(1.0 / 2.2);
            rgba8.push((tone(r) * 255.0) as u8);
            rgba8.push((tone(g) * 255.0) as u8);
            rgba8.push((tone(b) * 255.0) as u8);
            rgba8.push((a.clamp(0.0, 1.0) * 255.0) as u8);
        }
        image::save_buffer_with_format(
            format!("{example_dir}/noisy_input.png"),
            &rgba8,
            render_width,
            render_height,
            ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .expect("save noisy_input.png");
    }
    {
        let raw: &[u8] = readback_denoised
            .allocation
            .mapped_slice()
            .expect("readback_denoised mapped");
        image::save_buffer_with_format(
            format!("{example_dir}/denoised.png"),
            raw,
            target_width,
            target_height,
            ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .expect("save denoised.png");
    }

    println!(
        "DLSS-RR demo complete. Wrote noisy_input.png ({render_width}×{render_height}) and \
         denoised.png ({target_width}×{target_height}) to {example_dir}. Open both to see \
         the before/after."
    );

    // ===== Cleanup =====
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
        vk_mini_init.device.destroy_shader_module(shader_module, None);
    }
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, ubo);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback_noisy);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback_denoised);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, color_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, depth_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, normals_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, roughness_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, diffuse_albedo_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, specular_albedo_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, mv_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, out_img);
}

