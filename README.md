[![CI](https://github.com/iddm/nvngx-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/iddm/nvngx-rs/actions/workflows/ci.yml)
[![Crates](https://img.shields.io/crates/v/nvngx.svg)](https://crates.io/crates/nvngx)
[![Docs](https://docs.rs/nvngx/badge.svg)](https://docs.rs/nvngx)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# nvngx

A Rust wrapper over the NVIDIA NGX library.

The DLSS version used by this crate: [`3.10.5.3`](https://github.com/NVIDIA/DLSS/releases/tag/v310.5.3).

## Supported features

- DLSS (Super Sampling)
- DLSS-RR (Ray Reconstruction)
- DLSS-G (Frame Generation, including DLSS 4 multi-frame)

## Supported graphics APIs

- Vulkan (via [`ash:0.38`](https://crates.io/crates/ash/0.38.0+1.3.281) bindings).

## MSRV
1.71

## Platform support

As of writing NVIDIA only distributes Linux (under `glibc`) and Windows libraries for the `x86_64` architecture.  Use the following `cfg()`-conditional to ensure that this crate and your code only compile under those conditions, and implement a fallback otherwise:

```rust
cfg(all(target_arch = "x86_64", any(target_os = "windows", target_os = "linux")))
```

Such as in a `Cargo.toml`:

```toml
[target.'cfg(all(target_arch = "x86_64", any(target_os = "windows", target_os = "linux")))'.dependencies]
nvngx = "current.version"
```

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

## Running the examples

The repository ships five runnable examples under [crates/nvngx/examples/](crates/nvngx/examples/):

| Example                    | Demonstrates                                                                                           |
|----------------------------|--------------------------------------------------------------------------------------------------------|
| `upsample`                 | DLSS super-sampling on a still image (`baboon.png`).                                                   |
| `ray_reconstruction`       | DLSS-RR denoising a 1-spp Monte Carlo path-traced scene (sphere + plane + sky) over 32 jittered frames. Saves the noisy input and the denoised output side-by-side. |
| `ray_reconstruction_restir`| DLSS-RR with diffuse/specular hit-distance inputs (resource-set template).                             |
| `frame_generation`         | DLSS-G (DLSS 4 multi-frame aware) interpolating between two real frames of the panning baboon test image. |
| `combined`                 | All three features in one binary: DLSS upscaling on the baboon, then DLSS-RR with ReSTIR-style hit-distance inputs over 32 jittered frames of an orbiting ray-traced scene, then DLSS-G interpolating between the last two DLSS-RR outputs. |

### Hardware & driver

- **GPU:** NVIDIA RTX (Turing or newer for DLSS / DLSS-RR; Ada or newer for DLSS-G).
- **Driver:** a recent enough version for the requested feature. Each `supports_*()`
  call reports the minimum driver version when out of date, e.g.
  `Frame Generation feature requires a driver update. ... should be higher or equal to X.Y`.
- **OS / arch:** `x86_64` Linux (`glibc`) or Windows (see [Platform support](#platform-support)).

### Build prerequisites

- A C++ toolchain (the helpers in `crates/nvngx-sys/src/bindings.cpp` are compiled with `cc`).
- Vulkan headers:
  - **Linux:** install your distro's Vulkan headers (`vulkan-headers`, `libvulkan-dev`, etc.).
  - **Windows:** install the [Vulkan SDK](https://vulkan.lunarg.com/) and ensure the
    `VULKAN_SDK` environment variable is set; the build script reads it.
- The DLSS submodule must be checked out: `git submodule update --init --recursive`.

The Rust side then links `libnvsdk_ngx.a` (Linux) / `nvsdk_ngx_*.lib` (Windows) automatically.

### Runtime requirements

At runtime DLSS dynamically loads the per-feature snippet shared libraries from
`crates/nvngx-sys/DLSS/lib/`:

- Linux: `libnvidia-ngx-dlss.so.<ver>`, `libnvidia-ngx-dlssd.so.<ver>`, `libnvidia-ngx-dlssg.so.<ver>`
  in `Linux_x86_64/{rel,dev}/`.
- Windows: the matching `nvngx_dlss*.dll` files in `Windows_x86_64/{rel,dev}/`.

These need to be reachable by the dynamic loader **before launching the
example** — without this, NGX silently reports the affected feature as
unavailable even on supported hardware (this is the most common cause of
"Frame Generation not supported on this device" on a 4090). Two simplest
options:

```sh
# Linux (release snippet — pick `dev` for verbose snippet-side logging)
export LD_LIBRARY_PATH="$PWD/crates/nvngx-sys/DLSS/lib/Linux_x86_64/rel:$LD_LIBRARY_PATH"
```

```powershell
# Windows
$env:Path = "$pwd\crates\nvngx-sys\DLSS\lib\Windows_x86_64\rel;$env:Path"
```

Or copy / symlink the snippet next to the example binary in `target/debug/examples/`.

Note that DLSS / DLSS-RR may still work without the path being set (if a
system-wide DLSS install is present), while DLSS-G will not — so if only
Frame Generation fails, it is almost always this.

### Running

```sh
cargo run --example upsample
cargo run --example ray_reconstruction
cargo run --example ray_reconstruction_restir
cargo run --example frame_generation
cargo run --example combined
```

Each example writes its output PNG next to the example sources
(`crates/nvngx/examples/<name>/`).

To get verbose logging from the NGX runtime on Linux, set:

```sh
export __NGX_LOG_LEVEL=1
```

If a feature is reported as unavailable, double-check the driver version and that
you are running on the dGPU (e.g. on hybrid laptops force the NVIDIA GPU with
`__NV_PRIME_RENDER_OFFLOAD=1 __VK_LAYER_NV_optimus=NVIDIA_only`).

## License

MIT
