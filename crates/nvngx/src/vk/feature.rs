//! Describes NGX features and their parameters.

use super::*;

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

    fn release(&mut self) -> Result {
        unsafe { nvngx_sys::NVSDK_NGX_VULKAN_ReleaseFeature(self.0) }.into()
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

/// A type alias for feature parameter, like
/// [`nvngx_sys::NVSDK_NGX_Parameter_NumFrames`].
// pub type FeatureParameterName = std::ffi::CStr;
pub type FeatureParameterName = [u8];

/// Inserts a parameter into the debug map.
#[macro_export]
macro_rules! insert_parameter_debug {
    ($map:ident, $parameters:ident, ($key:path, bool),) => {
        if let Ok(value) = $parameters.get_bool($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value)
                );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, i32),) => {
        if let Ok(value) = $parameters.get_i32($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value),
            );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, u32),) => {
        if let Ok(value) = $parameters.get_u32($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value),
            );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, f32),) => {
        if let Ok(value) = $parameters.get_f32($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value),
            );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, u64),) => {
        if let Ok(value) = $parameters.get_u64($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value),
            );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, f64),) => {
        if let Ok(value) = $parameters.get_f64($key) {
            $map.insert(
                stringify!($key).to_owned(),
                format!("{:?}", value),
            );
        }
    };
    ($map:ident, $parameters:ident, ($key:path, $typ:ident), $(($next_key:path, $next_type:ident)),+,) => {
        $crate::insert_parameter_debug!($map, $parameters, ($key, $typ),);
        $crate::insert_parameter_debug!($map, $parameters, $(($next_key, $next_type)),+,);
    };
}

/// Feature parameters is a collection of parameters of a feature (ha!).
#[repr(transparent)]
pub struct FeatureParameters(pub(crate) *mut nvngx_sys::NVSDK_NGX_Parameter);

impl std::fmt::Debug for FeatureParameters {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[repr(transparent)]
        struct FeatureParametersDebugPrinter<'a>(&'a FeatureParameters);

        impl<'a> std::fmt::Debug for FeatureParametersDebugPrinter<'a> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                use std::collections::HashMap;

                let mut fmt = fmt.debug_struct("FeatureParameters");
                fmt.field("pointer_address", &self.0 .0);

                let populate_map = || -> HashMap<String, String> {
                    let mut map = HashMap::new();
                    let parameters = self.0;

                    // TODO: add more
                    insert_parameter_debug!(
                        map,
                        parameters,
                        (nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available, bool),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_Available,
                            bool
                        ),
                        (nvngx_sys::NVSDK_NGX_Parameter_InPainting_Available, bool),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_ImageSuperResolution_Available,
                            bool
                        ),
                        (nvngx_sys::NVSDK_NGX_Parameter_SlowMotion_Available, bool),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_VideoSuperResolution_Available,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_ImageSignalProcessing_Available,
                            bool
                        ),
                        (nvngx_sys::NVSDK_NGX_Parameter_DeepResolve_Available, bool),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_InPainting_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_ImageSuperResolution_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_SlowMotion_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_VideoSuperResolution_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_ImageSignalProcessing_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_DeepResolve_NeedsUpdatedDriver,
                            bool
                        ),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_FrameInterpolation_NeedsUpdatedDriver,
                            bool
                        ),
                        (nvngx_sys::NVSDK_NGX_Parameter_NumFrames, u32),
                        (nvngx_sys::NVSDK_NGX_Parameter_Scale, u32),
                        (nvngx_sys::NVSDK_NGX_Parameter_OptLevel, u32),
                        (nvngx_sys::NVSDK_NGX_Parameter_IsDevSnippetBranch, bool),
                        (
                            nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_ScaleFactor,
                            f32
                        ),
                    );
                    map
                };
                let map = populate_map();
                fmt.field("parameters", &map).finish()
            }
        }

        let debug = FeatureParametersDebugPrinter(self);
        fmt.debug_tuple("FeatureParameters").field(&debug).finish()
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
    pub fn new(&self) -> Result<Self> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        Result::from(unsafe { nvngx_sys::NVSDK_NGX_VULKAN_AllocateParameters(&mut ptr as *mut _) })
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
    pub fn get_capability_parameters() -> Result<Self> {
        let mut ptr: *mut nvngx_sys::NVSDK_NGX_Parameter = std::ptr::null_mut();
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut ptr as *mut _)
        })
        .map(|_| Self(ptr))
    }

    /// Sets the value for the parameter named `name` to be a
    /// type-erased (`void *`) pointer.
    pub fn set_ptr<T>(&self, name: &FeatureParameterName, ptr: *mut T) {
        unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_SetVoidPointer(
                self.0,
                name.as_ptr().cast(),
                ptr as *mut _,
            );
        }
    }

    /// Returns a type-erased pointer associated with the provided
    /// `name`.
    pub fn get_ptr(&self, name: &FeatureParameterName) -> Result<*mut std::ffi::c_void> {
        let mut ptr = std::ptr::null_mut();
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetVoidPointer(
                self.0,
                name.as_ptr().cast(),
                &mut ptr as *mut _,
            )
        })
        .map(|_| ptr)
    }

    /// Sets an [`bool`] value for the parameter named `name`. The
    /// [`bool`] type isn't supported in NGX, but the semantics - are. The
    /// boolean values are stored as integers with value `1` being
    /// `true` and `0` being `false`.
    pub fn set_bool(&self, name: &FeatureParameterName, value: bool) {
        unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_SetI(
                self.0,
                name.as_ptr().cast(),
                if value { 1 } else { 0 },
            )
        }
    }

    /// Returns a [`bool`] value of a parameter named `name`.
    /// The [`bool`] type isn't supported in NGX, but the semantics - are.
    /// The boolean values are stored as integers with value `1` being
    /// `true` and `0` being `false`.
    pub fn get_bool(&self, name: &FeatureParameterName) -> Result<bool> {
        let mut value = 0i32;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetI(self.0, name.as_ptr().cast(), &mut value as *mut _)
        })
        .map(|_| value == 1)
    }

    /// Sets an [`f32`] value for the parameter named `name`.
    pub fn set_f32(&self, name: &FeatureParameterName, value: f32) {
        unsafe { nvngx_sys::NVSDK_NGX_Parameter_SetF(self.0, name.as_ptr().cast(), value) }
    }

    /// Returns a [`f32`] value of a parameter named `name`.
    pub fn get_f32(&self, name: &FeatureParameterName) -> Result<f32> {
        let mut value = 0f32;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetF(self.0, name.as_ptr().cast(), &mut value as *mut _)
        })
        .map(|_| value)
    }

    /// Sets an [`u32`] value for the parameter named `name`.
    pub fn set_u32(&self, name: &FeatureParameterName, value: u32) {
        unsafe { nvngx_sys::NVSDK_NGX_Parameter_SetUI(self.0, name.as_ptr().cast(), value) }
    }

    /// Returns a [`u32`] value of a parameter named `name`.
    pub fn get_u32(&self, name: &FeatureParameterName) -> Result<u32> {
        let mut value = 0u32;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetUI(self.0, name.as_ptr().cast(), &mut value as *mut _)
        })
        .map(|_| value)
    }

    /// Sets an [`f64`] value for the parameter named `name`.
    pub fn set_f64(&self, name: &FeatureParameterName, value: f64) {
        unsafe { nvngx_sys::NVSDK_NGX_Parameter_SetD(self.0, name.as_ptr().cast(), value) }
    }

    /// Returns a [`f64`] value of a parameter named `name`.
    pub fn get_f64(&self, name: &FeatureParameterName) -> Result<f64> {
        let mut value = 0f64;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetD(self.0, name.as_ptr().cast(), &mut value as *mut _)
        })
        .map(|_| value)
    }

    /// Sets an [`i32`] value for the parameter named `name`.
    pub fn set_i32(&self, name: &FeatureParameterName, value: i32) {
        unsafe { nvngx_sys::NVSDK_NGX_Parameter_SetI(self.0, name.as_ptr().cast(), value) }
    }

    /// Returns a [`i32`] value of a parameter named `name`.
    pub fn get_i32(&self, name: &FeatureParameterName) -> Result<i32> {
        let mut value = 0i32;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetI(self.0, name.as_ptr().cast(), &mut value as *mut _)
        })
        .map(|_| value)
    }

    /// Sets an [`u64`] value for the parameter named `name`.
    pub fn set_u64(&self, name: &FeatureParameterName, value: u64) {
        unsafe { nvngx_sys::NVSDK_NGX_Parameter_SetULL(self.0, name.as_ptr().cast(), value) }
    }

    /// Returns a [`u64`] value of a parameter named `name`.
    pub fn get_u64(&self, name: &FeatureParameterName) -> Result<u64> {
        let mut value = 0u64;
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_Parameter_GetULL(
                self.0,
                name.as_ptr().cast(),
                &mut value as *mut _,
            )
        })
        .map(|_| value)
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// super sampling feature ([`nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available`]).
    pub fn supports_super_sampling(&self) -> Result<()> {
        if self.get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_NeedsUpdatedDriver)? {
            let major =
                self.get_u32(nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_MinDriverVersionMajor)?;
            let minor =
                self.get_u32(nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_MinDriverVersionMinor)?;
            return Err(nvngx_sys::Error::Other(format!("The SuperSampling feature requires a driver update. The driver version required should be higher or equal to {major}.{minor}")));
        }
        match self.get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available) {
            Ok(true) => Ok(()),
            Ok(false) => Err(nvngx_sys::Error::Other(
                "The SuperSampling feature isn't supported on this platform.".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// ray reconstruction feature ([`nvngx_sys::NVSDK_NGX_Feature::NVSDK_NGX_Feature_RayReconstruction`]).
    pub fn supports_ray_reconstruction(&self) -> Result<()> {
        if self
            .get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_NeedsUpdatedDriver)?
        {
            let major = self.get_u32(
                nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_MinDriverVersionMajor,
            )?;
            let minor = self.get_u32(
                nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_MinDriverVersionMinor,
            )?;
            return Err(nvngx_sys::Error::Other(format!("The Ray Reconstruction feature requires a driver update. The driver version required should be higher or equal to {major}.{minor}")));
        }
        match self.get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_Available) {
            Ok(true) => Ok(()),
            Ok(false) => Err(nvngx_sys::Error::Other(
                "The Ray Reconstruction feature isn't supported on this platform.".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// super sampling feature ([`nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available`]).
    pub fn supports_super_sampling_static() -> Result<()> {
        Self::get_capability_parameters()?.supports_super_sampling()
    }

    /// Returns [`Ok`] if the parameters claim to support the
    /// super sampling feature ([`nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_Available`]).
    pub fn supports_ray_reconstruction_static() -> Result<()> {
        Self::get_capability_parameters()?.supports_ray_reconstruction()
    }

    /// Returns [`true`] if the SuperSampling feature is initialised
    /// correctly.
    pub fn is_super_sampling_initialised(&self) -> bool {
        self.get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSampling_FeatureInitResult)
            .unwrap_or(false)
    }

    /// Returns [`true`] if the Ray Reconstruction feature is initialised
    /// correctly.
    pub fn is_ray_reconstruction_initialised(&self) -> bool {
        self.get_bool(nvngx_sys::NVSDK_NGX_Parameter_SuperSamplingDenoising_FeatureInitResult)
            .unwrap_or(false)
    }

    /// Deallocates the feature parameter set.
    fn release(&self) -> Result {
        unsafe { nvngx_sys::NVSDK_NGX_VULKAN_DestroyParameters(self.0) }.into()
    }
}

impl Drop for FeatureParameters {
    fn drop(&mut self) {
        if let Err(e) = self.release() {
            log::error!(
                "Couldn't release the feature parameter set: {:?}: {e}",
                self
            )
        }
    }
}

/// Describes a single NGX feature.
#[derive(Debug)]
pub struct Feature {
    /// The feature handle.
    pub handle: Rc<FeatureHandle>,
    /// The type of the feature.
    pub feature_type: nvngx_sys::NVSDK_NGX_Feature,
    /// The parameters of the feature.
    pub parameters: Rc<FeatureParameters>,
}

impl Feature {
    /// Creates a new feature.
    pub fn new(
        device: vk::Device,
        command_buffer: vk::CommandBuffer,
        feature_type: nvngx_sys::NVSDK_NGX_Feature,
        parameters: FeatureParameters,
    ) -> Result<Self> {
        let mut handle = FeatureHandle::new();
        Result::from(unsafe {
            nvngx_sys::NVSDK_NGX_VULKAN_CreateFeature1(
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
            Result::from(nvngx_sys::HELPERS_NGX_VULKAN_CREATE_DLSS_EXT1(
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
            Result::from(nvngx_sys::HELPERS_NGX_VULKAN_CREATE_DLSSD_EXT1(
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
            nvngx_sys::NVSDK_NGX_VULKAN_GetScratchBufferSize(
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
            nvngx_sys::NVSDK_NGX_VULKAN_EvaluateFeature_C(
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

/// Describes a set of NGX feature requirements.
#[repr(transparent)]
#[derive(Debug)]
pub struct FeatureRequirement(nvngx_sys::NVSDK_NGX_FeatureRequirement);
