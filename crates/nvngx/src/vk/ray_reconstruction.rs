//! The Ray Reconstruction feature.

use nvngx_sys::{
    vulkan::NVSDK_NGX_VK_DLSSD_Eval_Params, NVSDK_NGX_DLSSD_Create_Params,
    NVSDK_NGX_DLSS_Denoise_Mode, NVSDK_NGX_DLSS_Depth_Type, NVSDK_NGX_DLSS_Roughness_Mode,
};

use super::*;

// TODO: Turn back on
// impl From<SuperSamplingOptimalSettings> for RayReconstructionCreateParameters {
//     fn from(value: SuperSamplingOptimalSettings) -> Self {

//         Self::new(
//             value.render_width,
//             value.render_height,
//             value.target_width,
//             value.target_height,
//             Some(value.desired_quality_level),
//             None,
//             None,
//             None,
//         )
//     }
// }

/// Create parameters for the Ray Reconstruction feature.
#[repr(transparent)]
#[derive(Debug)]
pub struct RayReconstructionCreateParameters(pub(crate) NVSDK_NGX_DLSSD_Create_Params);

impl RayReconstructionCreateParameters {
    /// Creates a new set of create parameters for the SuperSampling
    /// feature.
    #[allow(clippy::too_many_arguments)] // Struct constructor
    pub fn new(
        render_width: u32,
        render_height: u32,
        target_width: u32,
        target_height: u32,
        quality_value: Option<NVSDK_NGX_PerfQuality_Value>,
        denoise_mode: Option<NVSDK_NGX_DLSS_Denoise_Mode>,
        roughness_mode: Option<NVSDK_NGX_DLSS_Roughness_Mode>,
        depth_type: Option<NVSDK_NGX_DLSS_Depth_Type>,
    ) -> Self {
        Self(NVSDK_NGX_DLSSD_Create_Params {
            InWidth: render_width,
            InHeight: render_height,
            InTargetWidth: target_width,
            InTargetHeight: target_height,
            // Equivalent to 0
            InPerfQualityValue: quality_value
                .unwrap_or(NVSDK_NGX_PerfQuality_Value::NVSDK_NGX_PerfQuality_Value_MaxPerf),
            InDenoiseMode: denoise_mode
                .unwrap_or(NVSDK_NGX_DLSS_Denoise_Mode::NVSDK_NGX_DLSS_Denoise_Mode_DLUnified),
            InRoughnessMode: roughness_mode
                .unwrap_or(NVSDK_NGX_DLSS_Roughness_Mode::NVSDK_NGX_DLSS_Roughness_Mode_Unpacked),
            InUseHWDepth: depth_type
                .unwrap_or(NVSDK_NGX_DLSS_Depth_Type::NVSDK_NGX_DLSS_Depth_Type_Linear),
            InFeatureCreateFlags: 0,
            InEnableOutputSubrects: false,
        })
    }
}

/// Similar to [`nvngx_sys::NVSDK_NGX_VK_DLSSD_Eval_Params`].
/// The Ray Reconstruction evaluation parameters.
///
#[derive(Debug)]
pub struct RayReconstructionEvaluationParameters {
    /// The vulkan resource which is an input to the evaluation
    /// parameters (for the upscaling).
    pub(crate) input_color_resource: NVSDK_NGX_Resource_VK,
    /// The vulkan resource which is the output of the evaluation,
    /// so the upscaled image.
    pub(crate) output_color_resource: NVSDK_NGX_Resource_VK,
    /// The depth buffer.
    pub(crate) depth_resource: NVSDK_NGX_Resource_VK,
    /// The motion vectors.
    pub(crate) motion_vectors_resource: NVSDK_NGX_Resource_VK,

    /// This member isn't visible, as it shouldn't be managed by
    /// the user of this struct. Instead, this struct provides an
    /// interface that populates this object and keeps it well-
    /// maintained.
    pub(crate) parameters: NVSDK_NGX_VK_DLSSD_Eval_Params,
}

impl Default for RayReconstructionEvaluationParameters {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl RayReconstructionEvaluationParameters {
    /// Creates a new set of evaluation parameters for SuperSampling.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color input parameter (the image to upscale).
    pub fn set_color_input(&mut self, description: VkImageResourceDescription) {
        self.input_color_resource = description.into();
        self.parameters.pInColor = std::ptr::addr_of_mut!(self.input_color_resource);
    }

    /// Sets the color output (the upscaled image) information.
    pub fn set_color_output(&mut self, description: VkImageResourceDescription) {
        self.output_color_resource = description.into();
        self.parameters.pInOutput = std::ptr::addr_of_mut!(self.output_color_resource);
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

    /// Returns the filled Ray Reconstruction parameters.
    pub(crate) fn get_rr_evaluation_parameters(&mut self) -> *mut NVSDK_NGX_VK_DLSSD_Eval_Params {
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

/// A helpful type alias to quickly mention "DLSS-RR".
pub type RRFeature<T> = RayReconstructionFeature<T>;

/// A Ray Reconstruction (or "DLSS-RR") feature.
#[derive(Debug)]
pub struct RayReconstructionFeature<T>
where
    T: FeatureHandleOps
        + FeatureParameterOps
        + FeatureOps<Device = vk::Device, CommandBuffer = vk::CommandBuffer>,
{
    feature: Feature<T>,
    parameters: RayReconstructionEvaluationParameters,
    rendering_resolution: vk::Extent2D,
    target_resolution: vk::Extent2D,
}

impl<T> RayReconstructionFeature<T>
where
    T: FeatureHandleOps
        + FeatureParameterOps
        + FeatureOps<Device = vk::Device, CommandBuffer = vk::CommandBuffer>,
{
    /// Creates a new Super Sampling feature.
    pub fn new(
        feature: Feature<T>,
        rendering_resolution: vk::Extent2D,
        target_resolution: vk::Extent2D,
    ) -> Result<Self> {
        if !feature.is_ray_reconstruction() {
            return Err(nvngx_sys::Error::Other(
                "Attempt to create a ray reconstruction feature with another feature.".to_owned(),
            ));
        }

        Ok(Self {
            feature,
            parameters: RayReconstructionEvaluationParameters::new(),
            rendering_resolution,
            target_resolution,
        })
    }

    /// Returns the inner feature object.
    pub fn get_inner(&self) -> &Feature<T> {
        &self.feature
    }

    /// Returns the inner feature object (mutable).
    pub fn get_inner_mut(&mut self) -> &mut Feature<T> {
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

    // /// Attempts to create the [`RayReconstructionFeature`] with the default
    // /// settings preset.
    // pub fn try_default() -> Result<Self> {
    //     let parameters = FeatureParameters::get_capability_parameters()?;
    //     Self::new(parameters)
    // }

    /// See [`FeatureParameters::is_super_sampling_initialised`].
    pub fn is_initialised(&self) -> bool {
        self.feature
            .get_parameters()
            .is_super_sampling_initialised()
    }

    /// Returns the evaluation parameters.
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut RayReconstructionEvaluationParameters {
        &mut self.parameters
    }

    /// Evaluates the feature.
    pub fn evaluate(&mut self, command_buffer: vk::CommandBuffer) -> Result {
        Result::from(unsafe {
            nvngx_sys::vulkan::HELPERS_NGX_VULKAN_EVALUATE_DLSSD_EXT(
                command_buffer,
                self.feature.handle.get_handle(),
                self.feature.parameters.get_params(),
                self.parameters.get_rr_evaluation_parameters(),
            )
        })
    }
}
