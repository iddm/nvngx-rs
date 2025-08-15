//! Describes and implements the interface for the DLSS feature.

use nvngx_sys::vulkan::NVSDK_NGX_VK_DLSS_Eval_Params;

use super::*;

/// A helpful type alias to quickly mention "DLSS".
pub type DlssFeature = SuperSamplingFeature;

// /// Optimal settings for the DLSS based on the desired quality level and
// /// resolution.
// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// pub struct SuperSamplingOptimalSettings {
//     /// The render width which the renderer must render to before
//     /// upscaling.
//     pub render_width: u32,
//     /// The render height which the renderer must render to before
//     /// upscaling.
//     pub render_height: u32,
//     // /// The target width desired, to which the SuperSampling feature
//     // /// will upscale to.
//     // pub target_width: u32,
//     // /// The target height desired, to which the SuperSampling feature
//     // /// will upscale to.
//     // pub target_height: u32,
//     /// The requested quality level.
//     pub desired_quality_level: NVSDK_NGX_PerfQuality_Value,
//     /// TODO:
//     pub dynamic_min_render_width: u32,
//     /// TODO:
//     pub dynamic_max_render_width: u32,
//     /// TODO:
//     pub dynamic_min_render_height: u32,
//     /// TODO:
//     pub dynamic_max_render_height: u32,
// }

// /// Create parameters for the SuperSampling feature.
// #[repr(transparent)]
// #[derive(Debug)]
// pub struct SuperSamplingCreateParameters(pub(crate) NVSDK_NGX_DLSS_Create_Params);

// impl SuperSamplingCreateParameters {
//     /// Creates a new set of create parameters for the SuperSampling
//     /// feature.
//     pub fn new(
//         render_width: u32,
//         render_height: u32,
//         target_width: u32,
//         target_height: u32,
//         quality_value: Option<NVSDK_NGX_PerfQuality_Value>,
//         flags: Option<NVSDK_NGX_DLSS_Feature_Flags>,
//     ) -> Self {
//         let mut params: NVSDK_NGX_DLSS_Create_Params = unsafe { std::mem::zeroed() };
//         params.Feature.InWidth = render_width;
//         params.Feature.InHeight = render_height;
//         params.Feature.InTargetWidth = target_width;
//         params.Feature.InTargetHeight = target_height;
//         if let Some(quality_value) = quality_value {
//             params.Feature.InPerfQualityValue = quality_value;
//         }
//         params.InFeatureCreateFlags = flags.map(|f| f.0).unwrap_or(0);
//         Self(params)
//     }

//     ///
//     pub fn from_settings(
//         target_width: u32,
//         target_height: u32,
//         desired_quality_level: Option<NVSDK_NGX_PerfQuality_Value>,
//         settings: SuperSamplingOptimalSettings,
//     ) -> Self {
//         Self::new(
//             settings.render_width,
//             settings.render_height,
//             target_width,
//             target_height,
//             desired_quality_level,
//             Some(
//                 NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_AutoExposure
//                     | NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_MVLowRes,
//             ),
//         )
//     }
// }

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

/// The SuperSampling evaluation parameters.
#[derive(Default, Debug)]
pub struct SuperSamplingEvaluationParameters {
    /// The vulkan resource which is an input to the evaluation
    /// parameters (for the upscaling).
    input_color_resource: NVSDK_NGX_Resource_VK,
    /// The vulkan resource which is the output of the evaluation,
    /// so the upscaled image.
    output_color_resource: NVSDK_NGX_Resource_VK,
    /// The depth buffer.
    depth_resource: NVSDK_NGX_Resource_VK,
    /// The motion vectors.
    motion_vectors_resource: NVSDK_NGX_Resource_VK,

    /// This member isn't visible, as it shouldn't be managed by
    /// the user of this struct. Instead, this struct provides an
    /// interface that populates this object and keeps it well-
    /// maintained.
    parameters: NVSDK_NGX_VK_DLSS_Eval_Params,
}

impl SuperSamplingEvaluationParameters {
    /// Creates a new set of evaluation parameters for SuperSampling.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color input parameter (the image to upscale).
    pub fn set_color_input(&mut self, description: VkImageResourceDescription) {
        self.input_color_resource = description.into();
        self.parameters.Feature.pInColor = std::ptr::addr_of_mut!(self.input_color_resource);
    }

    /// Sets the color output (the upscaled image) information.
    pub fn set_color_output(&mut self, description: VkImageResourceDescription) {
        self.output_color_resource = description.into();
        self.parameters.Feature.pInOutput = std::ptr::addr_of_mut!(self.output_color_resource);
    }

    /// Sets the motion vectors.
    /// In case the `scale` argument is omitted, the `1.0f32` scaling is
    /// used.
    pub fn set_motions_vectors(
        &mut self,
        description: VkImageResourceDescription,
        scale: Option<[f32; 2]>,
    ) {
        // 1.0f32 means no scaling (they are already in the pixel space).
        const DEFAULT_SCALING: [f32; 2] = [1.0f32, 1.0f32];

        self.motion_vectors_resource = description.into();
        let scales = scale.unwrap_or(DEFAULT_SCALING);
        self.parameters.pInMotionVectors = std::ptr::addr_of_mut!(self.motion_vectors_resource);
        self.parameters.InMVScaleX = scales[0];
        self.parameters.InMVScaleY = scales[1];
    }

    /// Sets the depth buffer.
    pub fn set_depth_buffer(&mut self, description: VkImageResourceDescription) {
        self.depth_resource = description.into();
        self.parameters.pInDepth = std::ptr::addr_of_mut!(self.depth_resource);
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
    pub(crate) fn get_dlss_evaluation_parameters(&mut self) -> *mut NVSDK_NGX_VK_DLSS_Eval_Params {
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
    rendering_resolution: vk::Extent2D,
    target_resolution: vk::Extent2D,
}

impl SuperSamplingFeature {
    /// Creates a new Super Sampling feature.
    pub fn new(
        feature: Feature,
        rendering_resolution: vk::Extent2D,
        target_resolution: vk::Extent2D,
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
    pub const fn get_rendering_resolution(&self) -> vk::Extent2D {
        self.rendering_resolution
    }

    /// Returns the target resolution (output resolution) of the
    /// image that the original image should be upscaled to.
    pub const fn get_target_resolution(&self) -> vk::Extent2D {
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
    pub fn evaluate(&mut self, command_buffer: vk::CommandBuffer) -> Result {
        Result::from(unsafe {
            nvngx_sys::vulkan::HELPERS_NGX_VULKAN_EVALUATE_DLSS_EXT(
                command_buffer,
                self.feature.handle.0,
                self.feature.parameters.0,
                self.parameters.get_dlss_evaluation_parameters(),
            )
        })
    }
}
