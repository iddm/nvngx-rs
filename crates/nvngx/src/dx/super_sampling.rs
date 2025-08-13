//! Directx bindings for supersampling

//! Describes and implements the interface for the DLSS feature.

use nvngx_sys::{
    directx::NVSDK_NGX_D3D12_DLSS_Eval_Params, NVSDK_NGX_Coordinates, NVSDK_NGX_Dimensions,
};
use windows::{core::Interface as _, Win32::Graphics::Direct3D12::ID3D12Resource};

use super::*;

/// A helpful type alias to quickly mention "DLSS".
pub type DlssFeature = SuperSamplingFeature;

// impl From<SuperSamplingOptimalSettings> for SuperSamplingCreateParameters {
//     fn from(value: SuperSamplingOptimalSettings) -> Self {
//         unsafe {
//             Self::new(
//                 value.render_width,
//                 value.render_height,
//                 value.target_width,
//                 value.target_height,
//                 Some(value.desired_quality_level),
//                 Some(
//                     NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_AutoExposure
//                         | NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_MVLowRes,
//                 ),
//             )
//         }
//     }
// }

// /// Only mandatory parameters for the SuperSampling feature evaluation.
// #[derive(Debug, derive_builder::Builder)]
// pub struct SuperSamplingEvaluationParametersSimple {
//     /// The feature evaluation parameters, specific to Vulkan.
//     feature_evaluation_parameters: nvngx_sys::NVSDK_NGX_VK_Feature_Eval_Params,
//     /// The depth information.
//     depth: nvngx_sys::NVSDK_NGX_Resource_VK,
//     /// The motion vectors.
//     motion_vectors: nvngx_sys::NVSDK_NGX_Resource_VK,
//     /// Jitter offset x.
//     jitter_offset_x: f32,
//     /// Jitter offset y.
//     jitter_offset_y: f32,
//     /// The dimensions of the viewport.
//     dimensions: nvngx_sys::NVSDK_NGX_Dimensions,
// }

// impl From<SuperSamplingEvaluationParametersSimple> for SuperSamplingEvaluationParameters {
//     fn from(value: SuperSamplingEvaluationParametersSimple) -> Self {
//         let mut params: nvngx_sys::NVSDK_NGX_VK_DLSS_Eval_Params = unsafe { std::mem::zeroed() };
//         params.Feature = value.feature_evaluation_parameters;
//         params.pInDepth = value.depth;
//         unsafe {
//             nvngx_sys::HELPERS_NVSDK_NGX_Create_ImageView_Resource_VK(imageView, image, subresourceRange, format, width, height, readWrite)
//         }
//         Self(params)
//     }
// }

// struct Desc<'a> {
//    pub input_color_resource: &'a ID3D12Resource,
// }

/// The SuperSampling evaluation parameters.
#[derive(Debug, Default)]
pub struct SuperSamplingEvaluationParameters {
    // WARNING: The ID3D12Resources are only cloned for lifetime purposes. Because windows-rs
    // implements Drop semantics. This is already not the case on the Vulkan side.
    // Technically this struct should not be stored inside the Feature struct, but passed anew
    // every time evaluate() is called (you typically want to call it with different textures
    // anyway if doing multibuffering...).

    // input_output: NVSDK_NGX_D3D12_Feature_Eval_Params,
    /// The vulkan resource which is an input to the evaluation
    /// parameters (for the upscaling).
    input_color_resource: Option<ID3D12Resource>,
    /// The vulkan resource which is the output of the evaluation,
    /// so the upscaled image.
    output_color_resource: Option<ID3D12Resource>,
    /// The depth buffer.
    depth_resource: Option<ID3D12Resource>,
    /// The motion vectors.
    motion_vectors_resource: Option<ID3D12Resource>,

    /// This member isn't visible, as it shouldn't be managed by
    /// the user of this struct. Instead, this struct provides an
    /// interface that populates this object and keeps it well-
    /// maintained.
    parameters: NVSDK_NGX_D3D12_DLSS_Eval_Params,
}

// impl Default for SuperSamplingEvaluationParameters {
//     fn default() -> Self {
//         unsafe { std::mem::zeroed() }
//     }
// }

impl SuperSamplingEvaluationParameters {
    /// Creates a new set of evaluation parameters for SuperSampling.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color input parameter (the image to upscale).
    pub fn set_color_input(
        &mut self,
        // feature_parameters: FeatureParameters,
        // eval_parameters: NVSDK_NGX_D3D12_DLSS_Eval_Params)
        resource: &ID3D12Resource,
    ) {
        // let name = std::ffi::CStr::from_bytes_with_nul(b"Color\0").unwrap();        unsafe {
        // NVSDK_NGX_Parameter_SetD3d12Resource(feature_parameters.0,
        // name.as_ptr(),
        // eval_parameters.Feature.pInColor) };

        self.input_color_resource = Some(resource.clone());
        self.parameters.Feature.pInColor = resource.as_raw().cast();
    }

    /// Sets the color output (the upscaled image) information.
    pub fn set_color_output(&mut self, resource: &ID3D12Resource) {
        self.output_color_resource = Some(resource.clone());
        self.parameters.Feature.pInOutput = resource.as_raw().cast();
    }

    /// Sets the motion vectors.
    /// In case the `scale` argument is omitted, the `1.0f32` scaling is
    /// used.
    pub fn set_motions_vectors(&mut self, resource: &ID3D12Resource, scale: Option<[f32; 2]>) {
        // 1.0f32 means no scaling (they are already in the pixel space).
        const DEFAULT_SCALING: [f32; 2] = [1.0f32, 1.0f32];

        self.motion_vectors_resource = Some(resource.clone());
        let scales = scale.unwrap_or(DEFAULT_SCALING);
        self.parameters.pInMotionVectors = resource.as_raw().cast();
        self.parameters.InMVScaleX = scales[0];
        self.parameters.InMVScaleY = scales[1];
    }

    /// Sets the depth buffer.
    pub fn set_depth_buffer(&mut self, resource: &ID3D12Resource) {
        self.depth_resource = Some(resource.clone());
        self.parameters.pInDepth = resource.as_raw().cast();
    }

    /// Sets the jitter offsets (like TAA).
    pub fn set_jitter_offsets(&mut self, x: f32, y: f32) {
        self.parameters.InJitterOffsetX = x;
        self.parameters.InJitterOffsetY = y;
    }

    /// Sets/unsets the reset flag.
    pub fn set_reset(&mut self, should_reset: bool) {
        self.parameters.InReset = if should_reset { 1 } else { 0 };
    }

    /// Sets the rendering dimensions.
    pub fn set_rendering_dimensions(
        &mut self,
        rendering_offset: [u32; 2],
        rendering_size: [u32; 2],
    ) {
        self.parameters.InColorSubrectBase = NVSDK_NGX_Coordinates {
            X: rendering_offset[0],
            Y: rendering_offset[1],
        };
        self.parameters.InDepthSubrectBase = NVSDK_NGX_Coordinates {
            X: rendering_offset[0],
            Y: rendering_offset[1],
        };
        self.parameters.InTranslucencySubrectBase = NVSDK_NGX_Coordinates {
            X: rendering_offset[0],
            Y: rendering_offset[1],
        };
        self.parameters.InMVSubrectBase = NVSDK_NGX_Coordinates {
            X: rendering_offset[0],
            Y: rendering_offset[1],
        };
        self.parameters.InRenderSubrectDimensions = NVSDK_NGX_Dimensions {
            Width: rendering_size[0],
            Height: rendering_size[1],
        };
    }

    /// Returns the filled DLSS parameters.
    pub(crate) fn get_dlss_evaluation_parameters(
        &mut self,
    ) -> *mut NVSDK_NGX_D3D12_DLSS_Eval_Params {
        std::ptr::addr_of_mut!(self.parameters)
    }

    // /// Returns an immutable reference to the color output.
    // pub fn get_color_output(&self) -> &VkImageResourceDescription {
    //     &self.color_output
    // }

    // /// Returns a mutable reference to the color output.
    // pub fn get_color_output_mut(&mut self) -> &mut VkImageResourceDescription {
    //     &mut self.color_output
    // }

    // /// Returns an immutable reference to the depth.
    // pub fn get_color(&self) -> &VkBufferResourceDescription {
    //     &self.depth
    // }

    // /// Returns a mutable reference to the depth.
    // pub fn get_color_mut(&mut self) -> &mut VkBufferResourceDescription {
    //     &mut self.depth
    // }
}

/// A SuperSamling (or "DLSS") feature.
#[derive(Debug)]
pub struct SuperSamplingFeature {
    feature: Feature,
    parameters: SuperSamplingEvaluationParameters,
    rendering_resolution: Extent2D,
    target_resolution: Extent2D,
}

impl SuperSamplingFeature {
    /// Creates a new Super Sampling feature.
    pub fn new(
        feature: Feature,
        rendering_resolution: Extent2D,
        target_resolution: Extent2D,
    ) -> Result<Self> {
        if !feature.is_super_sampling() {
            return Err(nvngx_sys::Error::Other(
                "Attempt to create a super sampling feature with another feature.".to_owned(),
            ));
        }

        Ok(Self {
            feature,
            parameters: SuperSamplingEvaluationParameters::new(),
            rendering_resolution,
            target_resolution,
        })
    }

    /// Returns the inner feature object.
    pub fn get_inner(&self) -> &Feature {
        &self.feature
    }

    /// Returns the inner feature object (mutable).
    pub fn get_inner_mut(&mut self) -> &mut Feature {
        &mut self.feature
    }

    /// Returns the rendering resolution (input resolution) of the
    /// image that needs to be upscaled to the `target_resolution`.
    pub const fn get_rendering_resolution(&self) -> Extent2D {
        self.rendering_resolution
    }

    /// Returns the target resolution (output resolution) of the
    /// image that the original image should be upscaled to.
    pub const fn get_target_resolution(&self) -> Extent2D {
        self.target_resolution
    }

    // /// Attempts to create the [`SuperSamplingFeature`] with the default
    // /// settings preset.
    // pub fn try_default() -> Result<Self> {
    //     let parameters = FeatureParameters::get_capability_parameters()?;
    //     Self::new(parameters)
    // }

    /// get feature handle
    pub fn get_feature_handle(&self) -> *mut nvngx_sys::NVSDK_NGX_Handle {
        self.feature.handle.0
    }

    /// See [`FeatureParameters::is_super_sampling_initialised`].
    pub fn is_initialised(&self) -> bool {
        self.feature
            .get_parameters()
            .is_super_sampling_initialised()
    }

    /// Returns the evaluation parameters.
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut SuperSamplingEvaluationParameters {
        &mut self.parameters
    }

    /// Evaluates the feature.
    pub fn evaluate(&mut self, command_buffer: &ID3D12GraphicsCommandList) -> Result {
        Result::from(unsafe {
            nvngx_sys::directx::HELPERS_NGX_D3D12_EVALUATE_DLSS_EXT(
                command_buffer.as_raw().cast(),
                self.feature.handle.0,
                self.feature.parameters.0,
                self.parameters.get_dlss_evaluation_parameters(),
            )
        })
    }
}
