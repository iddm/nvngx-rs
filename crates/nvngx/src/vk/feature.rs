//! Describes NGX features and their parameters.

use super::{super::ngx::FeatureParameters, *};

/// An NGX handle. Handle might be created and used by [`Feature::new()`].
#[repr(transparent)]
#[derive(Debug)]
pub struct FeatureHandle(pub(crate) *mut nvngx_sys::NVSDK_NGX_Handle);

impl Default for FeatureHandle {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

impl FeatureHandle {
    fn new() -> Self {
        Self::default()
    }

    // TODO: This should be unsafe, take Self (and write self.0=null()), or inlined into drop().
    fn release(&mut self) -> Result {
        unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_ReleaseFeature(self.0) }.into()
    }
}

impl Drop for FeatureHandle {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }

        if let Err(e) = self.release() {
            log::error!("Couldn't release the feature handle: {:?}: {e}", self)
        }
    }
}
impl FeatureParameters {
    /// Create a new feature parameter set.
    ///
    /// # NVIDIA documentation
    ///
    /// This interface allows allocating a simple parameter setup using named fields, whose
    /// lifetime the app must manage.
    /// For example one can set width by calling Set(NVSDK_NGX_Parameter_Denoiser_Width,100) or
    /// provide CUDA buffer pointer by calling Set(NVSDK_NGX_Parameter_Denoiser_Color,cudaBuffer)
    /// For more details please see sample code.
    /// Parameter maps output by NVSDK_NGX_AllocateParameters must NOT be freed using
    /// the free/delete operator; to free a parameter map
    /// output by NVSDK_NGX_AllocateParameters, NVSDK_NGX_DestroyParameters should be used.
    /// Unlike with NVSDK_NGX_GetParameters, parameter maps allocated with NVSDK_NGX_AllocateParameters
    /// must be destroyed by the app using NVSDK_NGX_DestroyParameters.
    /// Also unlike with NVSDK_NGX_GetParameters, parameter maps output by NVSDK_NGX_AllocateParameters
    /// do not come pre-populated with NGX capabilities and available features.
    /// To create a new parameter map pre-populated with such information, NVSDK_NGX_GetCapabilityParameters
    /// should be used.
    /// This function may return NVSDK_NGX_Result_FAIL_OutOfDate if an older driver, which
    /// does not support this API call is being used. In such a case, NVSDK_NGX_GetParameters
    /// may be used as a fallback.
    /// This function may only be called after a successful call into NVSDK_NGX_Init.
    pub fn create_vk(&self) -> Result<Self> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_AllocateParameters(&mut ptr as *mut _)
        })
        .map(|_| Self(ptr))
    }

    /// Get a feature parameter set populated with NGX and feature
    /// capabilities.
    ///
    /// # NVIDIA documentation
    ///
    /// This interface allows the app to create a new parameter map
    /// pre-populated with NGX capabilities and available features.
    /// The output parameter map can also be used for any purpose
    /// parameter maps output by NVSDK_NGX_AllocateParameters can be used for
    /// but it is not recommended to use NVSDK_NGX_GetCapabilityParameters
    /// unless querying NGX capabilities and available features
    /// due to the overhead associated with pre-populating the parameter map.
    /// Parameter maps output by NVSDK_NGX_GetCapabilityParameters must NOT be freed using
    /// the free/delete operator; to free a parameter map
    /// output by NVSDK_NGX_GetCapabilityParameters, NVSDK_NGX_DestroyParameters should be used.
    /// Unlike with NVSDK_NGX_GetParameters, parameter maps allocated with NVSDK_NGX_GetCapabilityParameters
    /// must be destroyed by the app using NVSDK_NGX_DestroyParameters.
    /// This function may return NVSDK_NGX_Result_FAIL_OutOfDate if an older driver, which
    /// does not support this API call is being used. This function may only be called
    /// after a successful call into NVSDK_NGX_Init.
    /// If NVSDK_NGX_GetCapabilityParameters fails with NVSDK_NGX_Result_FAIL_OutOfDate,
    /// NVSDK_NGX_GetParameters may be used as a fallback, to get a parameter map pre-populated
    /// with NGX capabilities and available features.
    pub fn vk_get_capability_parameters() -> Result<Self> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut ptr as *mut _)
        })
        .map(|_| Self(ptr))
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// super sampling feature ([`nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available`]).
    pub fn vk_supports_super_sampling_static() -> Result<()> {
        Self::vk_get_capability_parameters()?.supports_super_sampling()
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// super sampling feature ([`nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available`]).
    pub fn vk_supports_ray_reconstruction_static() -> Result<()> {
        Self::vk_get_capability_parameters()?.supports_ray_reconstruction()
    }

    /// Deallocates the feature parameter set.
    pub fn vk_release(&self) -> Result {
        unsafe { nvngx_sys::vulkan::NVSDK_NGX_VULKAN_DestroyParameters(self.0) }.into()
    }
}

/// Describes a single NGX feature.
#[derive(Debug)]
pub struct Feature {
    /// The feature handle.
    pub handle: Rc<FeatureHandle>,
    /// The type of the feature.
    pub feature_type: NVSDK_NGX_Feature,
    /// The parameters of the feature.
    pub parameters: Rc<FeatureParameters>,
}

impl Feature {
    /// Creates a new feature.
    pub fn new(
        device: vk::Device,
        command_buffer: vk::CommandBuffer,
        feature_type: NVSDK_NGX_Feature,
        parameters: FeatureParameters,
    ) -> Result<Self> {
        let mut handle = FeatureHandle::new();
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_CreateFeature1(
                device,
                command_buffer,
                feature_type,
                parameters.0,
                &mut handle.0 as *mut _,
            )
        })
        .map(|_| Self {
            handle: handle.into(),
            feature_type,
            parameters: parameters.into(),
        })
    }

    /// Creates a new SuperSampling feature.
    pub fn new_super_sampling(
        device: vk::Device,
        command_buffer: vk::CommandBuffer,
        parameters: FeatureParameters,
        mut super_sampling_create_parameters: SuperSamplingCreateParameters,
    ) -> Result<SuperSamplingFeature> {
        let feature_type = NVSDK_NGX_Feature::NVSDK_NGX_Feature_SuperSampling;
        let rendering_resolution = vk::Extent2D::default()
            .width(super_sampling_create_parameters.0.Feature.InWidth)
            .height(super_sampling_create_parameters.0.Feature.InHeight);
        let target_resolution = vk::Extent2D::default()
            .width(super_sampling_create_parameters.0.Feature.InTargetWidth)
            .height(super_sampling_create_parameters.0.Feature.InTargetHeight);
        unsafe {
            let mut handle = FeatureHandle::new();
            Result::from(nvngx_sys::vulkan::HELPERS_NGX_VULKAN_CREATE_DLSS_EXT1(
                device,
                command_buffer,
                1,
                1,
                &mut handle.0 as *mut _,
                parameters.0,
                &mut super_sampling_create_parameters.0 as *mut _,
            ))
            .and_then(|_| {
                SuperSamplingFeature::new(
                    Self {
                        handle: handle.into(),
                        feature_type,
                        parameters: parameters.into(),
                    },
                    rendering_resolution,
                    target_resolution,
                )
            })
        }
    }

    /// Creates the Frame Generation feature.
    pub fn new_frame_generation(
        device: vk::Device,
        command_buffer: vk::CommandBuffer,
        parameters: FeatureParameters,
    ) -> Result<Self> {
        let feature_type = NVSDK_NGX_Feature::NVSDK_NGX_Feature_FrameGeneration;
        Self::new(device, command_buffer, feature_type, parameters)
    }

    /// Creates the Ray Reconstruction feature.
    pub fn new_ray_reconstruction(
        device: vk::Device,
        command_buffer: vk::CommandBuffer,
        parameters: FeatureParameters,
        mut ray_reconstruction_create_parameters: RayReconstructionCreateParameters,
    ) -> Result<RayReconstructionFeature> {
        let feature_type = NVSDK_NGX_Feature::NVSDK_NGX_Feature_RayReconstruction;
        let rendering_resolution = vk::Extent2D::default()
            .width(ray_reconstruction_create_parameters.0.InWidth)
            .height(ray_reconstruction_create_parameters.0.InHeight);
        let target_resolution = vk::Extent2D::default()
            .width(ray_reconstruction_create_parameters.0.InTargetWidth)
            .height(ray_reconstruction_create_parameters.0.InTargetHeight);

        unsafe {
            let mut handle = FeatureHandle::new();
            Result::from(nvngx_sys::vulkan::HELPERS_NGX_VULKAN_CREATE_DLSSD_EXT1(
                device,
                command_buffer,
                1,
                1,
                &mut handle.0 as *mut _,
                parameters.0,
                &mut ray_reconstruction_create_parameters.0 as *mut _,
            ))
            .and_then(|_| {
                RayReconstructionFeature::new(
                    Self {
                        handle: handle.into(),
                        feature_type,
                        parameters: parameters.into(),
                    },
                    rendering_resolution,
                    target_resolution,
                )
            })
        }
    }

    /// Returns the parameters associated with this feature.
    pub fn get_parameters(&self) -> &FeatureParameters {
        &self.parameters
    }

    /// Returns the parameters associated with this feature.
    pub fn get_parameters_mut(&mut self) -> &mut FeatureParameters {
        Rc::get_mut(&mut self.parameters).unwrap()
    }

    /// Returns the type of this feature.
    pub fn get_feature_type(&self) -> NVSDK_NGX_Feature {
        self.feature_type
    }

    /// Returns [`true`] if this feature is the super sampling one.
    pub fn is_super_sampling(&self) -> bool {
        self.feature_type == NVSDK_NGX_Feature::NVSDK_NGX_Feature_SuperSampling
    }

    /// Returns [`true`] if this feature is the frame generation one.
    pub fn is_frame_generation(&self) -> bool {
        self.feature_type == NVSDK_NGX_Feature::NVSDK_NGX_Feature_FrameGeneration
    }

    /// Returns [`true`] if this feature is the ray reconstruction one.
    pub fn is_ray_reconstruction(&self) -> bool {
        self.feature_type == NVSDK_NGX_Feature::NVSDK_NGX_Feature_RayReconstruction
    }

    /// Returns the number of bytes needed for the scratch buffer for
    /// this feature.
    ///
    /// # NVIDIA documentation
    ///
    /// SDK needs a buffer of a certain size provided by the client in
    /// order to initialize AI feature. Once feature is no longer
    /// needed buffer can be released. It is safe to reuse the same
    /// scratch buffer for different features as long as minimum size
    /// requirement is met for all features. Please note that some
    /// features might not need a scratch buffer so return size of 0
    /// is completely valid.
    pub fn get_scratch_buffer_size(&self) -> Result<usize> {
        let mut size = 0usize;
        Result::from(unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_GetScratchBufferSize(
                self.feature_type,
                self.parameters.0 as _,
                &mut size as *mut _,
            )
        })
        .map(|_| size)
    }

    /// Evalutes the feature.
    ///
    /// # NVIDIA documentation
    ///
    /// Evaluates given feature using the provided parameters and
    /// pre-trained NN. Please note that for most features
    /// it can be benefitials to pass as many input buffers and parameters
    /// as possible (for example provide all render targets like color,
    /// albedo, normals, depth etc)
    pub fn evaluate(&self, command_buffer: vk::CommandBuffer) -> Result {
        unsafe {
            nvngx_sys::vulkan::NVSDK_NGX_VULKAN_EvaluateFeature_C(
                command_buffer,
                self.handle.0,
                self.parameters.0,
                Some(feature_progress_callback),
            )
        }
        .into()
    }
}

unsafe extern "C" fn feature_progress_callback(progress: f32, _should_cancel: *mut bool) {
    log::debug!("Feature evalution progress={progress}.");
}
