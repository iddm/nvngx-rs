//! The Frame Generation (DLSSG) feature.

use nvngx_sys::{
    NVSDK_NGX_DLSSG_Create_Params, NVSDK_NGX_DLSSG_Opt_Eval_Params,
    NVSDK_NGX_VK_DLSSG_Eval_Params,
};

use super::*;

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
/// [`nvngx_sys::NVSDK_NGX_DLSSG_Opt_Eval_Params`].
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
    /// The output interpolated frame.
    pub(crate) output_interp_frame_resource: NVSDK_NGX_Resource_VK,
    /// The output real frame.
    pub(crate) output_real_frame_resource: NVSDK_NGX_Resource_VK,

    /// The evaluation parameters.
    pub(crate) eval_params: NVSDK_NGX_VK_DLSSG_Eval_Params,
    /// The optional evaluation parameters (camera, scene, etc.).
    pub(crate) opt_eval_params: NVSDK_NGX_DLSSG_Opt_Eval_Params,
}

impl Default for FrameGenerationEvaluationParameters {
    fn default() -> Self {
        Self {
            backbuffer_resource: unsafe { std::mem::zeroed() },
            depth_resource: unsafe { std::mem::zeroed() },
            motion_vectors_resource: unsafe { std::mem::zeroed() },
            hudless_resource: unsafe { std::mem::zeroed() },
            ui_resource: unsafe { std::mem::zeroed() },
            output_interp_frame_resource: unsafe { std::mem::zeroed() },
            output_real_frame_resource: unsafe { std::mem::zeroed() },
            eval_params: NVSDK_NGX_VK_DLSSG_Eval_Params::default(),
            opt_eval_params: NVSDK_NGX_DLSSG_Opt_Eval_Params::default(),
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

    /// Sets the camera view-to-clip matrix (should NOT contain TAA jitter).
    pub fn set_camera_view_to_clip(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.cameraViewToClip = *matrix;
    }

    /// Sets the clip-to-camera-view matrix.
    pub fn set_clip_to_camera_view(&mut self, matrix: &[[f32; 4]; 4]) {
        self.opt_eval_params.clipToCameraView = *matrix;
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
