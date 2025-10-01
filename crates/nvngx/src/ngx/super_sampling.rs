//! Generic Supersampling code
use nvngx_sys::{
    NVSDK_NGX_DLSS_Create_Params, NVSDK_NGX_DLSS_Feature_Flags, NVSDK_NGX_PerfQuality_Value, Result,
};
use std::fmt::Debug;

/// Common trait for SuperSampling evaluation parameters across platforms
pub trait SuperSamplingEvaluationOps: Debug + Default {
    /// Generic Resources
    type ColorResource;
    /// Generic Resources
    type DepthResource;
    /// Generic Resources
    type MotionVectorResource;
    /// Generic Resources
    type CommandBuffer;

    /// Creates new evaluation parameters
    fn new() -> Self {
        Self::default()
    }

    /// Sets the color input parameter (the image to upscale)
    fn set_color_input(&mut self, resource: Self::ColorResource);

    /// Sets the color output (the upscaled image)
    fn set_color_output(&mut self, resource: Self::ColorResource);

    /// Sets the motion vectors with optional scaling
    fn set_motion_vectors(&mut self, resource: Self::MotionVectorResource, scale: Option<[f32; 2]>);

    /// Sets the depth buffer
    fn set_depth_buffer(&mut self, resource: Self::DepthResource);

    /// Sets the jitter offsets (like TAA)
    fn set_jitter_offsets(&mut self, x: f32, y: f32);

    /// Sets the rendering dimensions
    fn set_rendering_dimensions(&mut self, rendering_offset: [u32; 2], rendering_size: [u32; 2]);

    /// Evaluates the feature with the given command buffer and feature data
    fn evaluate(
        &mut self,
        command_buffer: Self::CommandBuffer,
        handle: *mut nvngx_sys::NVSDK_NGX_Handle,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
    ) -> Result<()>;
}

/// Generic SuperSampling feature that works across platforms
#[derive(Debug)]
pub struct SuperSamplingFeature<T, P>
where
    T: super::feature::FeatureHandleOps
        + super::feature::FeatureParameterOps
        + super::feature::FeatureOps,
    P: SuperSamplingEvaluationOps,
{
    feature: super::feature::Feature<T>,
    parameters: P,
    rendering_resolution: [u32; 2],
    target_resolution: [u32; 2],
}

impl<T, P> SuperSamplingFeature<T, P>
where
    T: super::feature::FeatureHandleOps
        + super::feature::FeatureParameterOps
        + super::feature::FeatureOps,
    P: SuperSamplingEvaluationOps,
{
    /// Creates a new Super Sampling feature
    pub fn new(
        feature: super::feature::Feature<T>,
        rendering_resolution: [u32; 2],
        target_resolution: [u32; 2],
    ) -> Result<Self, nvngx_sys::Error> {
        if !feature.is_super_sampling() {
            return Err(nvngx_sys::Error::Other(
                "Attempt to create a super sampling feature with another feature.".to_owned(),
            ));
        }

        Ok(Self {
            feature,
            parameters: P::new(),
            rendering_resolution,
            target_resolution,
        })
    }

    /// Returns the inner feature object
    pub fn get_inner(&self) -> &super::feature::Feature<T> {
        &self.feature
    }

    /// Returns the inner feature object (mutable)
    pub fn get_inner_mut(&mut self) -> &mut super::feature::Feature<T> {
        &mut self.feature
    }

    /// Returns the rendering resolution
    pub const fn get_rendering_resolution(&self) -> [u32; 2] {
        self.rendering_resolution
    }

    /// Returns the target resolution
    pub const fn get_target_resolution(&self) -> [u32; 2] {
        self.target_resolution
    }

    /// See FeatureParameters::is_super_sampling_initialised below
    pub fn is_initialised(&self) -> bool {
        self.feature
            .get_parameters()
            .is_super_sampling_initialised()
    }

    /// Returns the evaluation parameters
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut P {
        &mut self.parameters
    }

    /// Evaluates the feature
    pub fn evaluate(&mut self, command_buffer: P::CommandBuffer) -> Result<()> {
        self.parameters.evaluate(
            command_buffer,
            self.feature.handle.get_handle(),
            self.feature.parameters.get_params(),
        )
    }
}

/// Optimal settings for the DLSS based on the desired quality level and
/// resolution.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SuperSamplingOptimalSettings {
    /// The render width which the renderer must render to before
    /// upscaling.
    pub render_width: u32,
    /// The render height which the renderer must render to before
    /// upscaling.
    pub render_height: u32,
    // /// The target width desired, to which the SuperSampling feature
    // /// will upscale to.
    // pub target_width: u32,
    // /// The target height desired, to which the SuperSampling feature
    // /// will upscale to.
    // pub target_height: u32,
    /// The requested quality level.
    pub desired_quality_level: NVSDK_NGX_PerfQuality_Value,
    /// TODO:
    pub dynamic_min_render_width: u32,
    /// TODO:
    pub dynamic_max_render_width: u32,
    /// TODO:
    pub dynamic_min_render_height: u32,
    /// TODO:
    pub dynamic_max_render_height: u32,
}

impl SuperSamplingOptimalSettings {
    /// Returns a set of optimal settings for the desired parameter
    /// set, render dimensions and quality level.
    /// # Safety
    ///
    /// Parameters is a raw ptr and is therefor unsafe.
    pub unsafe fn get_optimal_settings(
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
        target_width: u32,
        target_height: u32,
        desired_quality_level: NVSDK_NGX_PerfQuality_Value,
    ) -> Result<Self> {
        let mut settings: Self = unsafe { std::mem::zeroed() };
        settings.desired_quality_level = desired_quality_level;
        // The sharpness is deprecated, should stay zero.
        let mut sharpness = 0.0f32;

        Result::from(unsafe {
            nvngx_sys::HELPERS_NGX_DLSS_GET_OPTIMAL_SETTINGS(
                parameters,
                target_width,
                target_height,
                desired_quality_level,
                &mut settings.render_width,
                &mut settings.render_height,
                &mut settings.dynamic_max_render_width,
                &mut settings.dynamic_max_render_height,
                &mut settings.dynamic_min_render_width,
                &mut settings.dynamic_min_render_height,
                &mut sharpness as *mut _,
            )
        })?;

        if settings.render_height == 0 || settings.render_width == 0 {
            return Err(nvngx_sys::Error::Other(format!(
                "The requested quality level isn't supported: {desired_quality_level:?}"
            )));
        }

        Ok(settings)
    }
}

/// Create parameters for the SuperSampling feature.
#[repr(transparent)]
#[derive(Debug)]
pub struct SuperSamplingCreateParameters(pub(crate) NVSDK_NGX_DLSS_Create_Params);

impl SuperSamplingCreateParameters {
    /// Creates a new set of create parameters for the SuperSampling
    /// feature.
    pub fn new(
        render_width: u32,
        render_height: u32,
        target_width: u32,
        target_height: u32,
        quality_value: Option<NVSDK_NGX_PerfQuality_Value>,
        flags: Option<NVSDK_NGX_DLSS_Feature_Flags>,
    ) -> Self {
        let mut params: NVSDK_NGX_DLSS_Create_Params = unsafe { std::mem::zeroed() };
        params.Feature.InWidth = render_width;
        params.Feature.InHeight = render_height;
        params.Feature.InTargetWidth = target_width;
        params.Feature.InTargetHeight = target_height;
        if let Some(quality_value) = quality_value {
            params.Feature.InPerfQualityValue = quality_value;
        }
        params.InFeatureCreateFlags = flags.map(|f| f.0).unwrap_or(0);
        Self(params)
    }
}
