//! The Frame Generation (DLSSG) feature.
//!
//! See `DLSS/doc/DLSS-FG Programming Guide.pdf` and the SDK headers
//! `nvsdk_ngx_helpers_dlssg_vk.h`, `nvsdk_ngx_params_dlssg.h`,
//! `nvsdk_ngx_defs_dlssg.h` for the canonical API description.

use nvngx_sys::{
    NVSDK_NGX_DLSSG_Create_Params, NVSDK_NGX_DLSSG_Opt_Eval_Params, NVSDK_NGX_PrecisionInfo,
    NVSDK_NGX_VK_DLSSG_Eval_Params,
};

use super::*;

bitflags::bitflags! {
    /// Bitmask version of [`nvngx_sys::NVSDK_NGX_DLSSG_EvalFlags`].
    /// Bindgen exposes the underlying SDK type as a single-variant
    /// `enum`, but the corresponding `DLSSG.EvalFlags` parameter is a
    /// bitfield in the C header.
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct DlssgEvalFlags: u32 {
        /// Default behaviour writes interpolated pixels inside the
        /// backbuffer extent and uninterpolated pixels outside; with
        /// this flag set, only the inside of the extent is updated.
        const UPDATE_ONLY_INSIDE_EXTENTS = 1 << 0;
    }
}

/// Create parameters for the Frame Generation feature.
#[repr(transparent)]
#[derive(Debug)]
pub struct FrameGenerationCreateParameters(pub(crate) NVSDK_NGX_DLSSG_Create_Params);

impl FrameGenerationCreateParameters {
    /// Creates a new set of create parameters for the Frame Generation
    /// feature.
    pub fn new(
        width: u32,
        height: u32,
        native_backbuffer_format: u32,
        render_width: Option<u32>,
        render_height: Option<u32>,
        dynamic_resolution_scaling: bool,
    ) -> Self {
        Self(NVSDK_NGX_DLSSG_Create_Params {
            Width: width,
            Height: height,
            NativeBackbufferFormat: native_backbuffer_format,
            RenderWidth: render_width.unwrap_or(width),
            RenderHeight: render_height.unwrap_or(height),
            DynamicResolutionScaling: dynamic_resolution_scaling,
        })
    }
}

/// The Frame Generation evaluation parameters.
///
/// Similar to [`nvngx_sys::NVSDK_NGX_VK_DLSSG_Eval_Params`] and
/// [`nvngx_sys::NVSDK_NGX_DLSSG_Opt_Eval_Params`]. Pointers in the
/// underlying SDK eval struct refer back to the resources owned by
/// this struct, so it must not be moved between
/// [`Self::set_*`](FrameGenerationEvaluationParameters::set_backbuffer)
/// calls and the matching evaluate call.
#[derive(Debug)]
pub struct FrameGenerationEvaluationParameters {
    /// The backbuffer resource.
    pub(crate) backbuffer_resource: NVSDK_NGX_Resource_VK,
    /// The depth buffer.
    pub(crate) depth_resource: NVSDK_NGX_Resource_VK,
    /// The motion vectors.
    pub(crate) motion_vectors_resource: NVSDK_NGX_Resource_VK,
    /// The HUD-less resource.
    pub(crate) hudless_resource: NVSDK_NGX_Resource_VK,
    /// The UI resource.
    pub(crate) ui_resource: NVSDK_NGX_Resource_VK,
    /// Color buffer with no post-processing applied.
    pub(crate) no_post_processing_color_resource: NVSDK_NGX_Resource_VK,
    /// Bidirectional distortion field.
    pub(crate) bidirectional_distortion_field_resource: NVSDK_NGX_Resource_VK,
    /// The output interpolated frame.
    pub(crate) output_interp_frame_resource: NVSDK_NGX_Resource_VK,
    /// The output real frame.
    pub(crate) output_real_frame_resource: NVSDK_NGX_Resource_VK,
    /// Output buffer that the snippet writes a single byte into to
    /// indicate that the host should not display the interpolated
    /// frame.
    pub(crate) output_disable_interpolation_resource: NVSDK_NGX_Resource_VK,

    /// The evaluation parameters.
    pub(crate) eval_params: NVSDK_NGX_VK_DLSSG_Eval_Params,
    /// The optional evaluation parameters (camera, scene, etc.).
    pub(crate) opt_eval_params: NVSDK_NGX_DLSSG_Opt_Eval_Params,
}

impl Default for FrameGenerationEvaluationParameters {
    fn default() -> Self {
        // `nvngx_sys`'s Default mirrors the C++ inline initialisers
        // (see the bindgen `.no_default(...)` note in
        // `nvngx-sys/build.rs`). `mvecScale` has no C++ inline
        // default; without overriding it here the snippet's helper
        // would scale every motion vector by zero.
        let opt_eval_params = NVSDK_NGX_DLSSG_Opt_Eval_Params {
            mvecScale: [1.0, 1.0],
            ..NVSDK_NGX_DLSSG_Opt_Eval_Params::default()
        };

        Self {
            backbuffer_resource: unsafe { std::mem::zeroed() },
            depth_resource: unsafe { std::mem::zeroed() },
            motion_vectors_resource: unsafe { std::mem::zeroed() },
            hudless_resource: unsafe { std::mem::zeroed() },
            ui_resource: unsafe { std::mem::zeroed() },
            no_post_processing_color_resource: unsafe { std::mem::zeroed() },
            bidirectional_distortion_field_resource: unsafe { std::mem::zeroed() },
            output_interp_frame_resource: unsafe { std::mem::zeroed() },
            output_real_frame_resource: unsafe { std::mem::zeroed() },
            output_disable_interpolation_resource: unsafe { std::mem::zeroed() },
            eval_params: NVSDK_NGX_VK_DLSSG_Eval_Params::default(),
            opt_eval_params,
        }
    }
}

impl FrameGenerationEvaluationParameters {
    /// Creates a new set of evaluation parameters for Frame Generation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the backbuffer (required).
    pub fn set_backbuffer(&mut self, description: VkImageResourceDescription) {
        self.backbuffer_resource = description.into();
        self.eval_params.pBackbuffer = std::ptr::addr_of_mut!(self.backbuffer_resource);
    }

    /// Sets the depth buffer (required).
    pub fn set_depth(&mut self, description: VkImageResourceDescription) {
        self.depth_resource = description.into();
        self.eval_params.pDepth = std::ptr::addr_of_mut!(self.depth_resource);
    }

    /// Sets the motion vectors (required).
    /// The `scale` controls normalization of motion vectors; if omitted,
    /// `[1.0, 1.0]` is used (pixel space).
    pub fn set_motion_vectors(
        &mut self,
        description: VkImageResourceDescription,
        scale: Option<[f32; 2]>,
    ) {
        const DEFAULT_SCALING: [f32; 2] = [1.0f32, 1.0f32];

        self.motion_vectors_resource = description.into();
        let scales = scale.unwrap_or(DEFAULT_SCALING);
        self.eval_params.pMVecs = std::ptr::addr_of_mut!(self.motion_vectors_resource);
        self.opt_eval_params.mvecScale = scales;
    }

    /// Sets the HUD-less resource (optional).
    pub fn set_hudless(&mut self, description: VkImageResourceDescription) {
        self.hudless_resource = description.into();
        self.eval_params.pHudless = std::ptr::addr_of_mut!(self.hudless_resource);
    }

    /// Sets the UI resource (optional).
    pub fn set_ui(&mut self, description: VkImageResourceDescription) {
        self.ui_resource = description.into();
        self.eval_params.pUI = std::ptr::addr_of_mut!(self.ui_resource);
    }

    /// Sets the no-post-processing color buffer (optional).
    pub fn set_no_post_processing_color(&mut self, description: VkImageResourceDescription) {
        self.no_post_processing_color_resource = description.into();
        self.eval_params.pNoPostProcessingColor =
            std::ptr::addr_of_mut!(self.no_post_processing_color_resource);
    }

    /// Sets the bidirectional distortion field resource (optional).
    pub fn set_bidirectional_distortion_field(
        &mut self,
        description: VkImageResourceDescription,
    ) {
        self.bidirectional_distortion_field_resource = description.into();
        self.eval_params.pBidirectionalDistortionField =
            std::ptr::addr_of_mut!(self.bidirectional_distortion_field_resource);
    }

    /// Sets the output interpolated frame (required).
    pub fn set_output_interpolated_frame(&mut self, description: VkImageResourceDescription) {
        self.output_interp_frame_resource = description.into();
        self.eval_params.pOutputInterpFrame =
            std::ptr::addr_of_mut!(self.output_interp_frame_resource);
    }

    /// Sets the output real frame (optional).
    pub fn set_output_real_frame(&mut self, description: VkImageResourceDescription) {
        self.output_real_frame_resource = description.into();
        self.eval_params.pOutputRealFrame =
            std::ptr::addr_of_mut!(self.output_real_frame_resource);
    }

    /// Sets the optional output buffer that the snippet writes to so
    /// it can ask the host to skip displaying the interpolated frame.
    /// Must be at least 4 bytes; the snippet writes `1` to the first
    /// byte if the interpolated frame should not be displayed.
    pub fn set_output_disable_interpolation(
        &mut self,
        description: VkImageResourceDescription,
    ) {
        self.output_disable_interpolation_resource = description.into();
        self.eval_params.pOutputDisableInterpolation =
            std::ptr::addr_of_mut!(self.output_disable_interpolation_resource);
    }

    /// Sets the multi-frame count and the current intermediate frame
    /// index (DLSS 4 multi-frame generation). For 2x, `count=1, index=1`.
    /// For 4x the host calls evaluate three times with `count=3` and
    /// `index=1..=3` (defaults to `(1, 1)`).
    pub fn set_multi_frame(&mut self, count: u32, index: u32) {
        self.opt_eval_params.multiFrameCount = count;
        self.opt_eval_params.multiFrameIndex = index;
    }

    /// Sets the camera view-to-clip matrix (should NOT contain TAA jitter).
    pub fn set_camera_view_to_clip(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.cameraViewToClip = *matrix;
    }

    /// Sets the clip-to-camera-view matrix.
    pub fn set_clip_to_camera_view(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.clipToCameraView = *matrix;
    }

    /// Sets the clip-to-lens-clip matrix used to describe lens
    /// distortion in clip space.
    pub fn set_clip_to_lens_clip(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.clipToLensClip = *matrix;
    }

    /// Sets the clip-to-previous-clip matrix.
    pub fn set_clip_to_prev_clip(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.clipToPrevClip = *matrix;
    }

    /// Sets the previous-clip-to-clip matrix.
    pub fn set_prev_clip_to_clip(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.prevClipToClip = *matrix;
    }

    /// Sets the clip space jitter offset.
    pub fn set_jitter_offset(&mut self, x: f32, y: f32) {
        self.opt_eval_params.jitterOffset = [x, y];
    }

    /// Sets the camera pinhole offset.
    pub fn set_camera_pinhole_offset(&mut self, x: f32, y: f32) {
        self.opt_eval_params.cameraPinholeOffset = [x, y];
    }

    /// Sets the camera position in world space.
    pub fn set_camera_position(&mut self, pos: [f32; 3]) {
        self.opt_eval_params.cameraPos = pos;
    }

    /// Sets the camera orientation vectors in world space.
    pub fn set_camera_vectors(&mut self, up: [f32; 3], right: [f32; 3], forward: [f32; 3]) {
        self.opt_eval_params.cameraUp = up;
        self.opt_eval_params.cameraRight = right;
        self.opt_eval_params.cameraFwd = forward;
    }

    /// Sets the camera near and far plane distances.
    pub fn set_camera_near_far(&mut self, near: f32, far: f32) {
        self.opt_eval_params.cameraNear = near;
        self.opt_eval_params.cameraFar = far;
    }

    /// Sets the camera field of view in radians.
    pub fn set_camera_fov(&mut self, fov: f32) {
        self.opt_eval_params.cameraFOV = fov;
    }

    /// Sets the camera aspect ratio (width / height).
    pub fn set_camera_aspect_ratio(&mut self, aspect: f32) {
        self.opt_eval_params.cameraAspectRatio = aspect;
    }

    /// Sets whether the color buffers are full HDR.
    pub fn set_color_buffers_hdr(&mut self, hdr: bool) {
        self.opt_eval_params.colorBuffersHDR = hdr;
    }

    /// Sets whether depth values are inverted (closer = higher value).
    pub fn set_depth_inverted(&mut self, inverted: bool) {
        self.opt_eval_params.depthInverted = inverted;
    }

    /// Sets whether camera motion is included in the motion vector buffer.
    pub fn set_camera_motion_included(&mut self, included: bool) {
        self.opt_eval_params.cameraMotionIncluded = included;
    }

    /// Sets whether the projection matrix is orthographic.
    pub fn set_ortho_projection(&mut self, ortho: bool) {
        self.opt_eval_params.orthoProjection = ortho;
    }

    /// Sets/unsets the reset flag (previous frame has no connection to
    /// the current one).
    pub fn set_reset(&mut self, reset: bool) {
        self.opt_eval_params.reset = reset;
    }

    /// Sets whether the application is not currently rendering game frames
    /// (e.g. paused in menu, playing video cut-scenes).
    pub fn set_not_rendering_game_frames(&mut self, not_rendering: bool) {
        self.opt_eval_params.notRenderingGameFrames = not_rendering;
    }

    /// Sets the sentinel value used to mark un-initialised motion
    /// vectors. Defaults to `0`.
    pub fn set_motion_vectors_invalid_value(&mut self, value: f32) {
        self.opt_eval_params.motionVectorsInvalidValue = value;
    }

    /// Sets whether the supplied motion vectors are already dilated.
    pub fn set_motion_vectors_dilated(&mut self, dilated: bool) {
        self.opt_eval_params.motionVectorsDilated = dilated;
    }

    /// Sets whether the snippet should run fullscreen menu detection.
    pub fn set_menu_detection_enabled(&mut self, enabled: bool) {
        self.opt_eval_params.menuDetectionEnabled = enabled;
    }

    /// Sets the precision info that describes how the
    /// bidirectional distortion field resource is encoded.
    pub fn set_bidirectional_distortion_field_precision(
        &mut self,
        is_low_precision: bool,
        bias: f32,
        scale: f32,
    ) {
        self.opt_eval_params.bidirectionalDistFieldPrecisionInfo = NVSDK_NGX_PrecisionInfo {
            IsLowPrecision: u32::from(is_low_precision),
            Bias: bias,
            Scale: scale,
        };
    }

    /// Sets the heuristic used to threshold the minimum linear depth
    /// difference between two screen-space objects. Defaults to 40.
    pub fn set_min_relative_linear_depth_object_separation(&mut self, value: f32) {
        self.opt_eval_params.minRelativeLinearDepthObjectSeparation = value;
    }

    /// Sets the subrect (origin + size) for the motion vectors resource.
    pub fn set_motion_vectors_subrect(&mut self, base: [u32; 2], size: [u32; 2]) {
        self.opt_eval_params.mvecsSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.mvecsSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Sets the subrect for the depth resource.
    pub fn set_depth_subrect(&mut self, base: [u32; 2], size: [u32; 2]) {
        self.opt_eval_params.depthSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.depthSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Sets the subrect for the HUD-less resource.
    pub fn set_hudless_subrect(&mut self, base: [u32; 2], size: [u32; 2]) {
        self.opt_eval_params.hudLessSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.hudLessSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Sets the subrect for the UI resource.
    pub fn set_ui_subrect(&mut self, base: [u32; 2], size: [u32; 2]) {
        self.opt_eval_params.uiSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.uiSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Sets the subrect for the bidirectional distortion field resource.
    pub fn set_bidirectional_distortion_field_subrect(
        &mut self,
        base: [u32; 2],
        size: [u32; 2],
    ) {
        self.opt_eval_params.bidirectionalDistFieldSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.bidirectionalDistFieldSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Sets the subrect for the backbuffer resource.
    pub fn set_backbuffer_subrect(&mut self, base: [u32; 2], size: [u32; 2]) {
        self.opt_eval_params.backbufferSubrectBase = NVSDK_NGX_Coordinates {
            X: base[0],
            Y: base[1],
        };
        self.opt_eval_params.backbufferSubrectSize = NVSDK_NGX_Dimensions {
            Width: size[0],
            Height: size[1],
        };
    }

    /// Returns a pointer to the evaluation parameters.
    pub(crate) fn get_eval_params(&mut self) -> *mut NVSDK_NGX_VK_DLSSG_Eval_Params {
        std::ptr::addr_of_mut!(self.eval_params)
    }

    /// Returns a pointer to the optional evaluation parameters.
    pub(crate) fn get_opt_eval_params(&mut self) -> *mut NVSDK_NGX_DLSSG_Opt_Eval_Params {
        std::ptr::addr_of_mut!(self.opt_eval_params)
    }
}

/// A helpful type alias to quickly mention "DLSSG".
pub type DlssgFeature = FrameGenerationFeature;

/// A Frame Generation (or "DLSSG") feature.
#[derive(Debug)]
pub struct FrameGenerationFeature {
    feature: Feature,
    parameters: FrameGenerationEvaluationParameters,
}

impl FrameGenerationFeature {
    /// Creates a new Frame Generation feature.
    pub fn new(feature: Feature) -> Result<Self> {
        if !feature.is_frame_generation() {
            return Err(nvngx_sys::Error::Other(
                "Attempt to create a frame generation feature with another feature.".to_owned(),
            ));
        }

        Ok(Self {
            feature,
            parameters: FrameGenerationEvaluationParameters::new(),
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

    /// See [`FeatureParameters::is_frame_generation_initialised`].
    pub fn is_initialised(&self) -> bool {
        self.feature
            .get_parameters()
            .is_frame_generation_initialised()
    }

    /// Returns the evaluation parameters.
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut FrameGenerationEvaluationParameters {
        &mut self.parameters
    }

    /// Sets the optional linearized depth scalar applied before
    /// algorithm processing (defaults to `1.0`). Lower values such as
    /// `0.1` can help with compressed input depth ranges.
    pub fn set_linearized_depth_scale(&self, scale: f32) {
        self.feature
            .get_parameters()
            .set_f32(nvngx_sys::NVSDK_NGX_DLSSG_Parameter_LinearizedDepth_Scale, scale);
    }

    /// Sets the optional `LinearizedDepth_NearFarPartition` heuristic
    /// (defaults to `600.0`).
    pub fn set_linearized_depth_near_far_partition(&self, value: f32) {
        self.feature.get_parameters().set_f32(
            nvngx_sys::NVSDK_NGX_DLSSG_Parameter_LinearizedDepth_NearFarPartition,
            value,
        );
    }

    /// Sets the eval-flags bitfield. Multiple flags may be OR'd
    /// together; pass [`DlssgEvalFlags::empty()`] to clear.
    pub fn set_eval_flags(&self, flags: DlssgEvalFlags) {
        self.feature
            .get_parameters()
            .set_u32(nvngx_sys::NVSDK_NGX_DLSSG_Parameter_EvalFlags, flags.bits());
    }

    /// Sets the optional 64-bit backbuffer frame id.
    pub fn set_backbuffer_frame_id(&self, id: u64) {
        self.feature
            .get_parameters()
            .set_u64(nvngx_sys::NVSDK_NGX_DLSSG_Parameter_BackbufferFrameID, id);
    }

    /// Returns the maximum number of intermediate frames the snippet
    /// can generate per real frame (DLSS 4). Returns `1` if the
    /// snippet does not advertise multi-frame support.
    pub fn multi_frame_count_max(&self) -> u32 {
        self.feature
            .get_parameters()
            .get_u32(nvngx_sys::NVSDK_NGX_DLSSG_Parameter_MultiFrameCountMax)
            .unwrap_or(1)
    }

    /// Evaluates the feature.
    pub fn evaluate(&mut self, command_buffer: vk::CommandBuffer) -> Result {
        Result::from(unsafe {
            nvngx_sys::HELPERS_NGX_VULKAN_EVALUATE_DLSSG(
                command_buffer,
                self.feature.handle.0,
                self.feature.parameters.0,
                self.parameters.get_eval_params(),
                self.parameters.get_opt_eval_params(),
            )
        })
    }
}
