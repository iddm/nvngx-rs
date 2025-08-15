//! Generic Supersampling code

use nvngx_sys::{
    NVSDK_NGX_DLSS_Create_Params, NVSDK_NGX_DLSS_Feature_Flags, NVSDK_NGX_PerfQuality_Value, Result,
};

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
    pub fn get_optimal_settings(
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
        target_width: u32,
        target_height: u32,
        desired_quality_level: NVSDK_NGX_PerfQuality_Value,
    ) -> Result<Self> {
        let mut settings: Self = unsafe { std::mem::zeroed() };
        settings.desired_quality_level = desired_quality_level;
        // The sharpness is deprecated, should stay zero.
        let mut sharpness = 0.0f32;
        unsafe {
            Result::from(nvngx_sys::HELPERS_NGX_DLSS_GET_OPTIMAL_SETTINGS(
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
            ))
        }?;

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

    /// doc
    pub fn from_settings(
        target_width: u32,
        target_height: u32,
        desired_quality_level: Option<NVSDK_NGX_PerfQuality_Value>,
        settings: SuperSamplingOptimalSettings,
    ) -> Self {
        Self::new(
            settings.render_width,
            settings.render_height,
            target_width,
            target_height,
            desired_quality_level,
            Some(
                NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_AutoExposure
                    | NVSDK_NGX_DLSS_Feature_Flags::NVSDK_NGX_DLSS_Feature_Flags_MVLowRes,
            ),
        )
    }
}
