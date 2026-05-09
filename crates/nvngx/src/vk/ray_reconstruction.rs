//! The Ray Reconstruction feature.
//!
//! See `DLSS/doc/DLSS-RR Integration Guide.pdf` and the SDK headers
//! `nvsdk_ngx_helpers_dlssd_vk.h`, `nvsdk_ngx_params_dlssd.h`,
//! `nvsdk_ngx_defs_dlssd.h` for the canonical API description.

use nvngx_sys::{
    NVSDK_NGX_DLSSD_Create_Params, NVSDK_NGX_DLSS_Denoise_Mode, NVSDK_NGX_DLSS_Depth_Type,
    NVSDK_NGX_DLSS_Roughness_Mode, NVSDK_NGX_ToneMapperType, NVSDK_NGX_VK_DLSSD_Eval_Params,
};

use super::*;

impl From<SuperSamplingOptimalSettings> for RayReconstructionCreateParameters {
    fn from(value: SuperSamplingOptimalSettings) -> Self {
        Self::new(
            value.render_width,
            value.render_height,
            value.target_width,
            value.target_height,
            Some(value.desired_quality_level),
            None,
            None,
            None,
        )
    }
}

/// Create parameters for the Ray Reconstruction feature.
#[repr(transparent)]
#[derive(Debug)]
pub struct RayReconstructionCreateParameters(pub(crate) nvngx_sys::NVSDK_NGX_DLSSD_Create_Params);

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

    /// OR-merges the given
    /// [`NVSDK_NGX_DLSS_Feature_Flags`](nvngx_sys::NVSDK_NGX_DLSS_Feature_Flags)
    /// bits into the existing flag set. The flags advertise
    /// properties of the host's input (HDR, jittered/low-resolution
    /// motion vectors, inverted depth, auto-exposure, alpha
    /// upscaling). Without at least the flags matching the host's
    /// actual G-buffer encoding, DLSS-RR `CreateFeature` typically
    /// fails with `Result_FAIL_InvalidParameter`.
    pub fn with_flags(mut self, flags: nvngx_sys::NVSDK_NGX_DLSS_Feature_Flags) -> Self {
        self.0.InFeatureCreateFlags |= flags.0;
        self
    }

    /// Enables per-evaluation output subrects. When set, evaluation
    /// can target a sub-region of the output image rather than the
    /// whole thing.
    pub fn with_output_subrects(mut self, enabled: bool) -> Self {
        self.0.InEnableOutputSubrects = enabled;
        self
    }
}

/// The Ray Reconstruction evaluation parameters.
///
/// Similar to [`nvngx_sys::NVSDK_NGX_VK_DLSSD_Eval_Params`]. Pointers
/// inside the underlying eval struct refer back to the resources
/// owned by this struct, so it must not be moved between
/// [`Self::set_*`](RayReconstructionEvaluationParameters::set_color_input)
/// calls and the matching evaluate call.
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
    /// The diffuse albedo.
    pub(crate) diffuse_albedo_resource: NVSDK_NGX_Resource_VK,
    /// The specular albedo.
    pub(crate) specular_albedo_resource: NVSDK_NGX_Resource_VK,
    /// The normals.
    pub(crate) normals_resource: NVSDK_NGX_Resource_VK,
    /// The roughness.
    pub(crate) roughness_resource: NVSDK_NGX_Resource_VK,
    /// The alpha channel.
    pub(crate) alpha_resource: NVSDK_NGX_Resource_VK,
    /// The output alpha channel.
    pub(crate) output_alpha_resource: NVSDK_NGX_Resource_VK,
    /// The transparency mask.
    pub(crate) transparency_mask_resource: NVSDK_NGX_Resource_VK,
    /// The exposure texture.
    pub(crate) exposure_texture_resource: NVSDK_NGX_Resource_VK,
    /// Mask used to bias the current color contribution per pixel.
    pub(crate) bias_current_color_mask_resource: NVSDK_NGX_Resource_VK,
    /// The diffuse hit distance.
    pub(crate) diffuse_hit_distance_resource: NVSDK_NGX_Resource_VK,
    /// The specular hit distance.
    pub(crate) specular_hit_distance_resource: NVSDK_NGX_Resource_VK,
    /// 3D motion vectors (when supplying full 3D MVs).
    pub(crate) motion_vectors_3d_resource: NVSDK_NGX_Resource_VK,
    /// Mask flagging pixels that contain particles.
    pub(crate) is_particle_mask_resource: NVSDK_NGX_Resource_VK,
    /// Mask covering pixels with animated textures.
    pub(crate) animated_texture_mask_resource: NVSDK_NGX_Resource_VK,
    /// High-resolution depth buffer.
    pub(crate) depth_high_res_resource: NVSDK_NGX_Resource_VK,
    /// View-space position buffer.
    pub(crate) position_view_space_resource: NVSDK_NGX_Resource_VK,
    /// Ray-tracing hit distance buffer.
    pub(crate) ray_tracing_hit_distance_resource: NVSDK_NGX_Resource_VK,
    /// Motion vectors of reflected objects (mirrors etc.).
    pub(crate) motion_vectors_reflections_resource: NVSDK_NGX_Resource_VK,
    /// Optional transparency layer color.
    pub(crate) transparency_layer_resource: NVSDK_NGX_Resource_VK,
    /// Optional transparency layer opacity.
    pub(crate) transparency_layer_opacity_resource: NVSDK_NGX_Resource_VK,
    /// Optional transparency layer motion vectors.
    pub(crate) transparency_layer_mvecs_resource: NVSDK_NGX_Resource_VK,
    /// Optional disocclusion mask.
    pub(crate) disocclusion_mask_resource: NVSDK_NGX_Resource_VK,

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

    /// Sets the diffuse albedo.
    pub fn set_diffuse_albedo(&mut self, description: VkImageResourceDescription) {
        self.diffuse_albedo_resource = description.into();
        self.parameters.pInDiffuseAlbedo =
            std::ptr::addr_of_mut!(self.diffuse_albedo_resource);
    }

    /// Sets the specular albedo.
    pub fn set_specular_albedo(&mut self, description: VkImageResourceDescription) {
        self.specular_albedo_resource = description.into();
        self.parameters.pInSpecularAlbedo =
            std::ptr::addr_of_mut!(self.specular_albedo_resource);
    }

    /// Sets the normals.
    pub fn set_normals(&mut self, description: VkImageResourceDescription) {
        self.normals_resource = description.into();
        self.parameters.pInNormals = std::ptr::addr_of_mut!(self.normals_resource);
    }

    /// Sets the roughness.
    pub fn set_roughness(&mut self, description: VkImageResourceDescription) {
        self.roughness_resource = description.into();
        self.parameters.pInRoughness = std::ptr::addr_of_mut!(self.roughness_resource);
    }

    /// Sets the alpha channel.
    pub fn set_alpha(&mut self, description: VkImageResourceDescription) {
        self.alpha_resource = description.into();
        self.parameters.pInAlpha = std::ptr::addr_of_mut!(self.alpha_resource);
    }

    /// Sets the output alpha channel.
    pub fn set_output_alpha(&mut self, description: VkImageResourceDescription) {
        self.output_alpha_resource = description.into();
        self.parameters.pInOutputAlpha =
            std::ptr::addr_of_mut!(self.output_alpha_resource);
    }

    /// Sets the transparency mask.
    pub fn set_transparency_mask(&mut self, description: VkImageResourceDescription) {
        self.transparency_mask_resource = description.into();
        self.parameters.pInTransparencyMask =
            std::ptr::addr_of_mut!(self.transparency_mask_resource);
    }

    /// Sets the exposure texture.
    pub fn set_exposure_texture(&mut self, description: VkImageResourceDescription) {
        self.exposure_texture_resource = description.into();
        self.parameters.pInExposureTexture =
            std::ptr::addr_of_mut!(self.exposure_texture_resource);
    }

    /// Sets the diffuse hit distance.
    pub fn set_diffuse_hit_distance(&mut self, description: VkImageResourceDescription) {
        self.diffuse_hit_distance_resource = description.into();
        self.parameters.pInDiffuseHitDistance =
            std::ptr::addr_of_mut!(self.diffuse_hit_distance_resource);
    }

    /// Sets the specular hit distance.
    pub fn set_specular_hit_distance(&mut self, description: VkImageResourceDescription) {
        self.specular_hit_distance_resource = description.into();
        self.parameters.pInSpecularHitDistance =
            std::ptr::addr_of_mut!(self.specular_hit_distance_resource);
    }

    /// Sets the bias-current-color mask.
    pub fn set_bias_current_color_mask(&mut self, description: VkImageResourceDescription) {
        self.bias_current_color_mask_resource = description.into();
        self.parameters.pInBiasCurrentColorMask =
            std::ptr::addr_of_mut!(self.bias_current_color_mask_resource);
    }

    /// Sets the 3D motion vectors resource. (Method spelling
    /// matches the existing [`Self::set_motions_vectors`].)
    pub fn set_motions_vectors_3d(&mut self, description: VkImageResourceDescription) {
        self.motion_vectors_3d_resource = description.into();
        self.parameters.pInMotionVectors3D =
            std::ptr::addr_of_mut!(self.motion_vectors_3d_resource);
    }

    /// Sets the particle-mask resource.
    pub fn set_is_particle_mask(&mut self, description: VkImageResourceDescription) {
        self.is_particle_mask_resource = description.into();
        self.parameters.pInIsParticleMask =
            std::ptr::addr_of_mut!(self.is_particle_mask_resource);
    }

    /// Sets the animated-texture mask resource.
    pub fn set_animated_texture_mask(&mut self, description: VkImageResourceDescription) {
        self.animated_texture_mask_resource = description.into();
        self.parameters.pInAnimatedTextureMask =
            std::ptr::addr_of_mut!(self.animated_texture_mask_resource);
    }

    /// Sets the high-resolution depth resource.
    pub fn set_depth_high_res(&mut self, description: VkImageResourceDescription) {
        self.depth_high_res_resource = description.into();
        self.parameters.pInDepthHighRes =
            std::ptr::addr_of_mut!(self.depth_high_res_resource);
    }

    /// Sets the view-space position resource.
    pub fn set_position_view_space(&mut self, description: VkImageResourceDescription) {
        self.position_view_space_resource = description.into();
        self.parameters.pInPositionViewSpace =
            std::ptr::addr_of_mut!(self.position_view_space_resource);
    }

    /// Sets the ray-tracing hit-distance resource (per-effect noise
    /// approximation).
    pub fn set_ray_tracing_hit_distance(&mut self, description: VkImageResourceDescription) {
        self.ray_tracing_hit_distance_resource = description.into();
        self.parameters.pInRayTracingHitDistance =
            std::ptr::addr_of_mut!(self.ray_tracing_hit_distance_resource);
    }

    /// Sets the motion vectors of reflected objects. (Method
    /// spelling matches the existing [`Self::set_motions_vectors`].)
    pub fn set_motions_vectors_reflections(
        &mut self,
        description: VkImageResourceDescription,
    ) {
        self.motion_vectors_reflections_resource = description.into();
        self.parameters.pInMotionVectorsReflections =
            std::ptr::addr_of_mut!(self.motion_vectors_reflections_resource);
    }

    /// Sets the optional transparency-layer color resource.
    pub fn set_transparency_layer(&mut self, description: VkImageResourceDescription) {
        self.transparency_layer_resource = description.into();
        self.parameters.pInTransparencyLayer =
            std::ptr::addr_of_mut!(self.transparency_layer_resource);
    }

    /// Sets the optional transparency-layer opacity resource.
    pub fn set_transparency_layer_opacity(&mut self, description: VkImageResourceDescription) {
        self.transparency_layer_opacity_resource = description.into();
        self.parameters.pInTransparencyLayerOpacity =
            std::ptr::addr_of_mut!(self.transparency_layer_opacity_resource);
    }

    /// Sets the optional transparency-layer motion vectors resource.
    pub fn set_transparency_layer_mvecs(&mut self, description: VkImageResourceDescription) {
        self.transparency_layer_mvecs_resource = description.into();
        self.parameters.pInTransparencyLayerMvecs =
            std::ptr::addr_of_mut!(self.transparency_layer_mvecs_resource);
    }

    /// Sets the optional disocclusion-mask resource.
    pub fn set_disocclusion_mask(&mut self, description: VkImageResourceDescription) {
        self.disocclusion_mask_resource = description.into();
        self.parameters.pInDisocclusionMask =
            std::ptr::addr_of_mut!(self.disocclusion_mask_resource);
    }

    /// Sets the pre-exposure value. Defaults to `1.0` if set to `0.0`.
    pub fn set_pre_exposure(&mut self, value: f32) {
        self.parameters.InPreExposure = value;
    }

    /// Sets the exposure scale. Defaults to `1.0` if set to `0.0`.
    pub fn set_exposure_scale(&mut self, value: f32) {
        self.parameters.InExposureScale = value;
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

    /// Sets the time elapsed since the previous frame, in milliseconds.
    /// Used to scale denoising/anti-aliasing strength based on motion.
    pub fn set_frame_time_delta_msec(&mut self, msec: f32) {
        self.parameters.InFrameTimeDeltaInMsec = msec;
    }

    /// Sets the tone-mapper type used by the application.
    pub fn set_tone_mapper_type(&mut self, ty: NVSDK_NGX_ToneMapperType) {
        self.parameters.InToneMapperType = ty;
    }

    /// Sets the debug-indicator inversion flags (used to flip the
    /// developer overlay axes).
    pub fn set_indicator_invert_axes(&mut self, invert_x: bool, invert_y: bool) {
        self.parameters.InIndicatorInvertXAxis = i32::from(invert_x);
        self.parameters.InIndicatorInvertYAxis = i32::from(invert_y);
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

    /// Sets the output subrect base (used together with
    /// `InEnableOutputSubrects`).
    pub fn set_output_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InOutputSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the diffuse-albedo input.
    pub fn set_diffuse_albedo_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InDiffuseAlbedoSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the specular-albedo input.
    pub fn set_specular_albedo_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InSpecularAlbedoSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the normals input.
    pub fn set_normals_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InNormalsSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the roughness input.
    pub fn set_roughness_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InRoughnessSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the alpha input.
    pub fn set_alpha_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InAlphaSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the alpha output.
    pub fn set_output_alpha_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InOutputAlphaSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Sets the per-resource subrect base for the bias-current-color mask.
    pub fn set_bias_current_color_subrect_base(&mut self, base: [u32; 2]) {
        self.parameters.InBiasCurrentColorSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
    }

    /// Returns the filled Ray Reconstruction parameters.
    pub(crate) fn get_rr_evaluation_parameters(
        &mut self,
    ) -> *mut nvngx_sys::NVSDK_NGX_VK_DLSSD_Eval_Params {
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
pub type RRFeature = RayReconstructionFeature;

/// A Ray Reconstruction (or "DLSS-RR") feature.
#[derive(Debug)]
pub struct RayReconstructionFeature {
    feature: Feature,
    parameters: RayReconstructionEvaluationParameters,
    rendering_resolution: vk::Extent2D,
    target_resolution: vk::Extent2D,
}

impl RayReconstructionFeature {
    /// Creates a new Super Sampling feature.
    pub fn new(
        feature: Feature,
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

    // /// Attempts to create the [`RayReconstructionFeature`] with the default
    // /// settings preset.
    // pub fn try_default() -> Result<Self> {
    //     let parameters = FeatureParameters::get_capability_parameters()?;
    //     Self::new(parameters)
    // }

    /// See [`FeatureParameters::is_ray_reconstruction_initialised`].
    pub fn is_initialised(&self) -> bool {
        self.feature
            .get_parameters()
            .is_ray_reconstruction_initialised()
    }

    /// Returns the evaluation parameters.
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut RayReconstructionEvaluationParameters {
        &mut self.parameters
    }

    /// Evaluates the feature.
    pub fn evaluate(&mut self, command_buffer: vk::CommandBuffer) -> Result {
        Result::from(unsafe {
            nvngx_sys::HELPERS_NGX_VULKAN_EVALUATE_DLSSD_EXT(
                command_buffer,
                self.feature.handle.0,
                self.feature.parameters.0,
                self.parameters.get_rr_evaluation_parameters(),
            )
        })
    }
}
