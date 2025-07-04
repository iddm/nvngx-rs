[![CI](https://github.com/iddm/nvngx-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/iddm/nvngx-rs/actions/workflows/ci.yml)
[![Crates](https://img.shields.io/crates/v/nvngx-rs.svg)](https://crates.io/crates/nvngx)
[![Docs](https://docs.rs/nvngx-rs/badge.svg)](https://docs.rs/nvngx)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# NGX-rs

A Rust wrapper over the NVIDIA NGX library.

The DLSS version used by this crate: [`3.10.1.0`](https://github.com/NVIDIA/DLSS/releases/tag/v310.1.0).

## Supported features

- DLSS
- Ray Reconstruction

## Supported graphics APIs

- Vulkan (only the [`ash`](https://crates.io/crates/ash) backend).

## MSRV
1.65

## DLSS integration example

One can have something like that:

```rust
#[derive(Debug)]
pub struct Ngx {
    super_sampling_feature: ngx::SuperSamplingFeature,
    system: ngx::System,
}

impl Ngx {
    /// Creates a new NVIDIA NGX module instance with the super
    /// sampling feature.
    pub fn new(
        logical_device: &LogicalDevice,
        command_pool: &CommandPool,
        extent: vk::Extent2D,
        dlss_profile: crate::config::DlssProfile,
    ) -> Result<Self> {
        let path = std::path::Path::new("/tmp/").canonicalize().unwrap();

        let physical_device = logical_device.get_physical().get_handle();
        let instance = &logical_device.get_instance();
        let system = ngx::System::new(
            None,
            env!("CARGO_PKG_VERSION"),
            &path,
            instance.get_entry(),
            &instance.get(),
            physical_device,
            logical_device.handle(),
        )?;

        let capability_parameters = ngx::vk::FeatureParameters::get_capability_parameters()?;
        log::debug!("NGX capability parameters: {capability_parameters:#?}");

        if let Err(e) = capability_parameters.supports_super_sampling() {
            return Err(e.into());
        }

        log::debug!("DLSS is supported, great!");

        if !capability_parameters.is_super_sampling_initialised() {
            return Err("Super sampling couldn't initialise.".into());
        }

        log::debug!("DLSS initialised correctly!");

        let optimal_settings = ngx::vk::SuperSamplingOptimalSettings::get_optimal_settings(
            &capability_parameters,
            extent.width,
            extent.height,
            dlss_profile.into(),
        )?;

        let command_buffer = command_pool.allocate_primary_command_buffer_scoped()?;
        command_buffer.set_label("NGXCreateSuperSampling")?;

        command_buffer.begin_recording()?;

        let super_sampling_feature = system.create_super_sampling_feature(
            command_buffer.get(),
            capability_parameters,
            optimal_settings.into(),
        )?;

        command_buffer.finish_recording()?;
        command_buffer.submit_and_wait_and_clear()?;

        Ok(Self {
            super_sampling_feature,
            system,
        })
    }
}
```

After that, to render, one need to properly prepare the feature, before
issuing a draw call. For example, (using the `ash` crate for Vulkan):

```rust
fn update_upscaling_configuration_parameters(&mut self) -> Result {
    let jitter = self.get_pixel_jitter();
    let dlss = self.ngx.super_sampling_feature;
    let parameters = dlss.get_evaluation_parameters_mut();

    // This is where you render your main scene to. Shouldn't contain
    // any text, just the scene, shouldn't be post-processed.
    parameters.set_color_input(self.storage_image.as_ref().into());

    let mut output: ngx::VkImageResourceDescription = self.upscaled_image.as_ref().into();
    output.set_writable();
    /// The image to which the DLSS will upscale to. Should be of the
    /// output resolution (rendering resolution).
    parameters.set_color_output(output);

    // An image of motion vectors.
    parameters.set_motions_vectors(
        self.motion_vectors_image.as_ref().into(),
        // Use the default scaling.
        None,
    );

    /// Jitter is optional, but should provide better results. Note that
    /// it must also be applied to the camera, and so the motion vectors
    /// should include it.
    parameters.set_jitter_offsets(jitter.x, jitter.y);

    /// The depth buffer.
    parameters.set_depth_buffer(self.depth_image.as_ref().into());
    let rendering_size = [
        self.storage_image.get_extent().width,
        self.storage_image.get_extent().height,
    ];

    // The dimensions of the output image.
    parameters.set_rendering_dimensions([0, 0], rendering_size);

    Ok(())
}
```

## License

MIT
