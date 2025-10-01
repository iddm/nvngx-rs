//! Directx bindings for supersampling

//! Describes and implements the interface for the DLSS feature.

use nvngx_sys::{
    directx::NVSDK_NGX_D3D12_DLSS_Eval_Params, NVSDK_NGX_Coordinates, NVSDK_NGX_Dimensions,
};
use windows::{core::Interface as _, Win32::Graphics::Direct3D12::ID3D12Resource};

use crate::ngx::SuperSamplingEvaluationOps;

use super::*;

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

impl crate::ngx::super_sampling::SuperSamplingEvaluationOps for SuperSamplingEvaluationParameters {
    type ColorResource = ID3D12Resource;
    type DepthResource = ID3D12Resource;
    type MotionVectorResource = ID3D12Resource;
    type CommandBuffer = ID3D12GraphicsCommandList;
    

    /// Creates a new set of evaluation parameters for SuperSampling.
    fn new() -> Self {
        Self::default()
    }

    /// Sets the color input parameter (the image to upscale).
    fn set_color_input(&mut self, resource: Self::ColorResource) {
        self.input_color_resource = Some(resource.clone());
        self.parameters.Feature.pInColor = resource.as_raw().cast();
    }

    /// Sets the color output (the upscaled image) information.
    fn set_color_output(&mut self, resource: Self::ColorResource) {
        self.output_color_resource = Some(resource.clone());
        self.parameters.Feature.pInOutput = resource.as_raw().cast();
    }

    /// Sets the motion vectors.
    /// In case the `scale` argument is omitted, the `1.0f32` scaling is
    /// used.
    fn set_motion_vectors(
        &mut self,
        resource: Self::MotionVectorResource,
        scale: Option<[f32; 2]>,
    ) {
        // 1.0f32 means no scaling (they are already in the pixel space).
        const DEFAULT_SCALING: [f32; 2] = [1.0f32, 1.0f32];

        self.motion_vectors_resource = Some(resource.clone());
        let scales = scale.unwrap_or(DEFAULT_SCALING);
        self.parameters.pInMotionVectors = resource.as_raw().cast();
        self.parameters.InMVScaleX = scales[0];
        self.parameters.InMVScaleY = scales[1];
    }

    /// Sets the depth buffer.
    fn set_depth_buffer(&mut self, resource: Self::DepthResource) {
        self.depth_resource = Some(resource.clone());
        self.parameters.pInDepth = resource.as_raw().cast();
    }

    /// Sets the jitter offsets (like TAA).
    fn set_jitter_offsets(&mut self, x: f32, y: f32) {
        self.parameters.InJitterOffsetX = x;
        self.parameters.InJitterOffsetY = y;
    }

    /// Sets the rendering dimensions.
    fn set_rendering_dimensions(&mut self, rendering_offset: [u32; 2], rendering_size: [u32; 2]) {
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

    // Applies the actual upscaling algorithm
    fn evaluate(
        &mut self,
        command_buffer: Self::CommandBuffer,
        handle: *mut nvngx_sys::NVSDK_NGX_Handle,
        parameters: *mut nvngx_sys::NVSDK_NGX_Parameter,
    ) -> Result<()> {
        nvngx_sys::Result::from(unsafe {
            nvngx_sys::directx::HELPERS_NGX_D3D12_EVALUATE_DLSS_EXT(
                command_buffer.as_raw().cast(),
                handle,
                parameters,
                std::ptr::addr_of_mut!(self.parameters),
            )
        })
    }
}

/// A SuperSamling (or "DLSS") feature.
#[derive(Debug)]
pub struct SuperSamplingFeature<T>
where
    T: crate::ngx::FeatureHandleOps + crate::ngx::FeatureParameterOps + crate::ngx::FeatureOps,
{
    feature: crate::ngx::Feature<T>,
    parameters: SuperSamplingEvaluationParameters,
    rendering_resolution: [u32; 2],
    target_resolution: [u32; 2],
}

impl<T> SuperSamplingFeature<T>
where
    T: crate::ngx::FeatureHandleOps + crate::ngx::FeatureParameterOps + crate::ngx::FeatureOps,
{
    /// Creates a new Super Sampling feature.
    pub fn new(
        feature: crate::ngx::Feature<T>,
        rendering_resolution: [u32; 2],
        target_resolution: [u32; 2],
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
    pub fn get_inner(&self) -> &crate::ngx::Feature<T> {
        &self.feature
    }

    /// Returns the inner feature object (mutable).
    pub fn get_inner_mut(&mut self) -> &mut crate::ngx::Feature<T> {
        &mut self.feature
    }

    /// Returns the rendering resolution (input resolution) of the
    /// image that needs to be upscaled to the `target_resolution`.
    pub const fn get_rendering_resolution(&self) -> [u32; 2] {
        self.rendering_resolution
    }

    /// Returns the target resolution (output resolution) of the
    /// image that the original image should be upscaled to.
    pub const fn get_target_resolution(&self) -> [u32; 2] {
        self.target_resolution
    }

    // /// Attempts to create the [`SuperSamplingFeature`] with the default
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
    pub fn get_evaluation_parameters_mut(&mut self) -> &mut SuperSamplingEvaluationParameters {
        &mut self.parameters
    }

    /// Returns the filled DLSS parameters.
    pub(crate) fn get_dlss_evaluation_parameters(
        &mut self,
    ) -> *mut NVSDK_NGX_D3D12_DLSS_Eval_Params {
        std::ptr::addr_of_mut!(self.parameters.parameters)
    }

    /// Evaluates the feature.
    pub fn evaluate(&mut self, command_buffer: &ID3D12GraphicsCommandList) -> Result {
        Result::from(unsafe {
            nvngx_sys::directx::HELPERS_NGX_D3D12_EVALUATE_DLSS_EXT(
                command_buffer.as_raw().cast(),
                self.feature.handle.get_handle(),
                self.feature.parameters.get_params(),
                self.get_dlss_evaluation_parameters(),
            )
        })
    }
}
