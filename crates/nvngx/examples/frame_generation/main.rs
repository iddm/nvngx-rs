//! DLSS-G (Frame Generation) demo.
//!
//! Renders two real frames of the baboon test image panning to the
//! right by 32 pixels, then asks DLSS-G to interpolate the in-between
//! frame. The interpolated PNG should show the baboon at roughly half
//! that offset; if DLSS-G silently no-ops, the interpolated frame
//! matches one of the endpoints instead.
//!
//! Outputs three PNGs in this directory:
//!   - `prev_real.png`         — frame N (baboon at offset 0)
//!   - `interpolated.png`      — DLSS-G interpolated output
//!   - `current_real.png`      — frame N+1 (baboon at offset PAN_X)
//!
//! Drag the three through any viewer that supports a slideshow / scrub
//! to see motion.

#[path = "../common/mod.rs"]
mod common;
use common::{allocations, imgops, vk_mini_init};

use ash::vk;
use image::ColorType;
use nvngx::FrameGenerationFeature;

/// How far (in pixels) the baboon translates to the right between
/// the two real frames. The interpolated frame is expected at PAN_X/2.
///
/// Value is constrained to a small whole pixel count so the MV
/// image can stay R16G16_SFLOAT without saturating (max f16 is
/// ~65504; non-integer fractions also lose precision in f16).
const PAN_X: i32 = 32;
const _: () = assert!(
    PAN_X.unsigned_abs() <= 4096,
    "PAN_X must fit in f16 with comfortable headroom — increase MV format if you need more.",
);

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
    if let Err(e) = capability_parameters.supports_frame_generation() {
        eprintln!("Frame Generation not supported on this device: {e}");
        std::process::exit(1);
    }

    // 1) Load the baboon. We pad the canvas around it so the baboon can
    //    actually translate and stay on-screen — DLSS-G won't have
    //    anything sensible to do with motion vectors that point off the
    //    backbuffer.
    let (baboon_rgba, baboon_w, baboon_h) = allocations::load_png_rgba8(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/upsample/baboon.png"
    ));
    let canvas_w = baboon_w + (PAN_X.unsigned_abs() * 2);
    let canvas_h = baboon_h;

    // 2) Create the DLSS-G feature.
    let create_params = nvngx::vk::FrameGenerationCreateParameters::new(
        canvas_w,
        canvas_h,
        vk::Format::R8G8B8A8_UNORM.as_raw() as u32,
        None,
        None,
        false,
    );
    let mut fg: nvngx_sys::Result<FrameGenerationFeature> =
        Err(nvngx::sys::Error::Other("Not initialized".to_string()));
    vk_mini_init
        .record_and_submit(|cb, _| {
            fg = system.create_frame_generation_feature(cb, capability_parameters, create_params);
        })
        .unwrap();
    let mut fg = fg.expect("create Frame Generation feature");
    println!(
        "DLSS-G initialised. multiFrameCountMax = {} ({}x mode max)",
        fg.multi_frame_count_max(),
        fg.multi_frame_count_max() + 1
    );

    let mut allocator = vk_mini_init.get_allocator();

    // 3) Resources.
    // Persistent backbuffer that gets rewritten between the two evaluates.
    let mut backbuffer = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        canvas_w,
        canvas_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::STORAGE,
    );
    // Per-pixel motion vector image. Reset to 0 for frame 1, set to
    // (PAN_X, 0) for frame 2. R16G16_SFLOAT in pixel space (default).
    let mut mv_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        canvas_w,
        canvas_h,
        vk::Format::R16G16_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
    );
    // Constant depth — there is no real geometry. DLSS-G still needs a
    // depth resource for its disocclusion logic.
    let mut depth_img = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        canvas_w,
        canvas_h,
        vk::Format::R32_SFLOAT,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
    );
    // Output: interpolated frame written by DLSS-G.
    let mut output_interp = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        canvas_w,
        canvas_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE,
    );
    // Optional output: real frame DLSS-G may scribble debug info onto.
    let mut output_real = allocations::create_image_optimal(
        &vk_mini_init.device,
        &mut allocator,
        canvas_w,
        canvas_h,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE,
    );

    // 4) Stage the baboon pixels (RGB → RGBA conversion already done by
    //    `load_png_rgba8`).
    let baboon_byte_len = (baboon_w * baboon_h * 4) as u64;
    let mut staging = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        baboon_byte_len,
        vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    staging
        .allocation
        .mapped_slice_mut()
        .expect("staging mapped")
        .copy_from_slice(baboon_rgba.as_raw());

    // Three readback buffers: prev real frame, interpolated frame,
    // current real frame.
    let canvas_byte_len = (canvas_w * canvas_h * 4) as u64;
    let readback_prev = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        canvas_byte_len,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );
    let readback_interp = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        canvas_byte_len,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );
    let readback_current = allocations::create_buffer(
        &vk_mini_init.device,
        &mut allocator,
        canvas_byte_len,
        vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuToCpu,
    );

    // 5) Record both DLSS-G evaluates back-to-back into one submission.
    vk_mini_init
        .record_and_submit(|cb, dev| {
            // Move every resource to GENERAL. Use the all-encompassing
            // TRANSFER stage so that subsequent transfer ops (clears
            // and buffer-to-image copies) chain correctly.
            for img in [
                &mut backbuffer,
                &mut mv_img,
                &mut depth_img,
                &mut output_interp,
                &mut output_real,
            ] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }

            // Constant depth and zero motion for frame 1.
            imgops::clear_color_image(dev, cb, depth_img.image, [0.5, 0.0, 0.0, 0.0]);
            imgops::clear_color_image(dev, cb, mv_img.image, [0.0, 0.0, 0.0, 0.0]);

            let subresource = imgops::default_subresource_range();
            let mk_desc = |img: &allocations::ImageAllocation, format, writable| {
                nvngx::vk::VkImageResourceDescription {
                    image_view: img.view,
                    image: img.image,
                    subresource_range: subresource,
                    format,
                    width: canvas_w,
                    height: canvas_h,
                    mode: if writable {
                        nvngx::vk::VkResourceMode::Writable
                    } else {
                        nvngx::vk::VkResourceMode::Readable
                    },
                }
            };

            // ===== Frame 1 (previous): baboon at offset 0 =====
            imgops::clear_color_image(dev, cb, backbuffer.image, [0.05, 0.05, 0.07, 1.0]);
            // Clear and copy are both TRANSFER_WRITE; without a
            // barrier they race. Re-issue a no-layout-change WAW
            // barrier on the backbuffer.
            backbuffer.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_buffer_to_image_with_offset(
                dev,
                cb,
                staging.buffer,
                backbuffer.image,
                [0, 0],
                [baboon_w, baboon_h],
            );

            // Snapshot the real previous-frame backbuffer for the user.
            backbuffer.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                backbuffer.image,
                readback_prev.buffer,
                canvas_w,
                canvas_h,
            );

            let eval = fg.get_evaluation_parameters_mut();
            eval.set_backbuffer(mk_desc(&backbuffer, vk::Format::R8G8B8A8_UNORM, false));
            eval.set_depth(mk_desc(&depth_img, vk::Format::R32_SFLOAT, false));
            eval.set_motion_vectors(
                mk_desc(&mv_img, vk::Format::R16G16_SFLOAT, false),
                None, // 1.0 scale (pixel-space MVs)
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

            // Identity camera. Background panning is encoded in MVs.
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
            eval.set_camera_aspect_ratio(canvas_w as f32 / canvas_h as f32);
            eval.set_color_buffers_hdr(false);
            eval.set_depth_inverted(false);
            eval.set_camera_motion_included(true);
            eval.set_multi_frame(1, 1);
            eval.set_reset(true); // first frame: no temporal history yet

            fg.evaluate(cb).expect("DLSS-G evaluate (frame 1)");

            // The snippet records its own internal pipeline barriers
            // we can't observe; mark each touched image as
            // post-snippet so the next `image_barrier` call sources
            // from a sane state.
            for img in [
                &mut backbuffer,
                &mut depth_img,
                &mut mv_img,
                &mut output_interp,
                &mut output_real,
            ] {
                img.current_stage = vk::PipelineStageFlags2::ALL_COMMANDS;
                img.current_access = vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE;
                img.current_layout = vk::ImageLayout::GENERAL;
            }

            // Sync: DLSS-G's compute reads from backbuffer/MV/depth must
            // complete before we overwrite them for frame 2.
            for img in [&mut backbuffer, &mut mv_img] {
                img.image_barrier(
                    dev,
                    cb,
                    vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::ImageLayout::GENERAL,
                );
            }

            // The output_interp from frame 1 is undefined (no previous
            // frame existed); we discard it.

            // ===== Frame 2 (current): baboon at offset PAN_X =====
            imgops::clear_color_image(dev, cb, backbuffer.image, [0.05, 0.05, 0.07, 1.0]);
            // Same WAW issue as frame 1.
            backbuffer.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_buffer_to_image_with_offset(
                dev,
                cb,
                staging.buffer,
                backbuffer.image,
                [PAN_X, 0],
                [baboon_w, baboon_h],
            );
            // MVs: every pixel moved by (PAN_X, 0) between previous and current.
            // Background pixels are technically static, but using a uniform
            // field is the common case for a fully-static-camera pan and is
            // what DLSS-G expects when there is no per-object information.
            imgops::clear_color_image(dev, cb, mv_img.image, [PAN_X as f32, 0.0, 0.0, 0.0]);

            // Snapshot the real current-frame backbuffer.
            backbuffer.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                backbuffer.image,
                readback_current.buffer,
                canvas_w,
                canvas_h,
            );

            let eval = fg.get_evaluation_parameters_mut();
            // Same resources as before; just refresh and disable reset.
            eval.set_backbuffer(mk_desc(&backbuffer, vk::Format::R8G8B8A8_UNORM, false));
            eval.set_depth(mk_desc(&depth_img, vk::Format::R32_SFLOAT, false));
            eval.set_motion_vectors(
                mk_desc(&mv_img, vk::Format::R16G16_SFLOAT, false),
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
            eval.set_reset(false);

            fg.evaluate(cb).expect("DLSS-G evaluate (frame 2)");

            // Mark each image as "snippet just touched it" again.
            for img in [
                &mut backbuffer,
                &mut depth_img,
                &mut mv_img,
                &mut output_interp,
                &mut output_real,
            ] {
                img.current_stage = vk::PipelineStageFlags2::ALL_COMMANDS;
                img.current_access = vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE;
                img.current_layout = vk::ImageLayout::GENERAL;
            }

            // Read back the interpolated frame.
            output_interp.image_barrier(
                dev,
                cb,
                vk::PipelineStageFlags2::CLEAR | vk::PipelineStageFlags2::COPY,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::GENERAL,
            );
            imgops::copy_image_to_buffer(
                dev,
                cb,
                output_interp.image,
                readback_interp.buffer,
                canvas_w,
                canvas_h,
            );
        })
        .unwrap();

    // 6) Save all three PNGs.
    let example_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/frame_generation");
    let save = |buf: &allocations::BufferAllocation, name: &str| {
        let mapped = buf.allocation.mapped_slice().expect("readback mapped");
        image::save_buffer_with_format(
            format!("{example_dir}/{name}"),
            mapped,
            canvas_w,
            canvas_h,
            ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .unwrap_or_else(|_| panic!("save {name}"));
    };
    save(&readback_prev, "prev_real.png");
    save(&readback_interp, "interpolated.png");
    save(&readback_current, "current_real.png");

    println!(
        "DLSS-G demo complete. Wrote prev_real.png / interpolated.png / current_real.png \
         to {example_dir}. The baboon moves from x=0 → x={PAN_X}; the interpolated \
         frame should show it near x={}.",
        PAN_X / 2,
    );

    // 7) Cleanup
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, staging);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback_prev);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback_interp);
    allocations::destroy_buffer(&vk_mini_init.device, &mut allocator, readback_current);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, backbuffer);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, mv_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, depth_img);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, output_interp);
    allocations::destroy_image(&vk_mini_init.device, &mut allocator, output_real);
}
