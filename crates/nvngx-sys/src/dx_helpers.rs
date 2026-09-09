//! Pure Rust reimplementations of the `static inline` DX12 helper functions/macros
//! from the NVIDIA NGX SDK headers (`nvsdk_ngx_helpers.h`, `nvsdk_ngx_helpers_dlssd_d3d.h`).

use crate::{dx::*, NVSDK_NGX_GBufferType::*, *};

/// Equivalent of `NGX_D3D12_CREATE_DLSS_EXT` from `nvsdk_ngx_helpers.h`.
///
/// # Safety
///
/// All pointer parameters must be valid.
pub unsafe fn d3d12_create_dlss_ext(
    in_cmd_list: *mut ID3D12GraphicsCommandList,
    in_creation_node_mask: u32,
    in_visibility_node_mask: u32,
    pp_out_handle: *mut *mut NVSDK_NGX_Handle,
    p_in_params: *mut NVSDK_NGX_Parameter,
    p_in_dlss_create_params: *mut NVSDK_NGX_DLSS_Create_Params,
) -> NVSDK_NGX_Result {
    let params = &*p_in_dlss_create_params;

    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_CreationNodeMask.as_ptr().cast(),
        in_creation_node_mask,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_VisibilityNodeMask.as_ptr().cast(),
        in_visibility_node_mask,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_Width.as_ptr().cast(),
        params.Feature.InWidth,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_Height.as_ptr().cast(),
        params.Feature.InHeight,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_OutWidth.as_ptr().cast(),
        params.Feature.InTargetWidth,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_OutHeight.as_ptr().cast(),
        params.Feature.InTargetHeight,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_PerfQualityValue.as_ptr().cast(),
        params.Feature.InPerfQualityValue as i32,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Feature_Create_Flags
            .as_ptr()
            .cast(),
        params.InFeatureCreateFlags as i32,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Enable_Output_Subrects
            .as_ptr()
            .cast(),
        if params.InEnableOutputSubrects { 1 } else { 0 },
    );

    NVSDK_NGX_D3D12_CreateFeature(
        in_cmd_list,
        NVSDK_NGX_Feature::NVSDK_NGX_Feature_SuperSampling,
        p_in_params,
        pp_out_handle,
    )
}

/// Equivalent of `NGX_D3D12_EVALUATE_DLSS_EXT` from `nvsdk_ngx_helpers.h`.
///
/// # Safety
///
/// All pointer parameters must be valid.
pub unsafe fn d3d12_evaluate_dlss_ext(
    in_cmd_list: *mut ID3D12GraphicsCommandList,
    p_in_handle: *mut NVSDK_NGX_Handle,
    p_in_params: *mut NVSDK_NGX_Parameter,
    p_in_dlss_eval_params: *mut NVSDK_NGX_D3D12_DLSS_Eval_Params,
) -> NVSDK_NGX_Result {
    let p = &*p_in_dlss_eval_params;

    macro_rules! set_ptr {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetVoidPointer(p_in_params, $name.as_ptr().cast(), $val as *mut _);
        };
    }
    macro_rules! set_f {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetF(p_in_params, $name.as_ptr().cast(), $val);
        };
    }
    macro_rules! set_ui {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetUI(p_in_params, $name.as_ptr().cast(), $val);
        };
    }
    macro_rules! set_i {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetI(p_in_params, $name.as_ptr().cast(), $val);
        };
    }

    set_ptr!(NVSDK_NGX_Parameter_Color, p.Feature.pInColor);
    set_ptr!(NVSDK_NGX_Parameter_Output, p.Feature.pInOutput);
    set_ptr!(NVSDK_NGX_Parameter_Depth, p.pInDepth);
    set_ptr!(NVSDK_NGX_Parameter_MotionVectors, p.pInMotionVectors);
    set_f!(NVSDK_NGX_Parameter_Jitter_Offset_X, p.InJitterOffsetX);
    set_f!(NVSDK_NGX_Parameter_Jitter_Offset_Y, p.InJitterOffsetY);
    set_f!(NVSDK_NGX_Parameter_Sharpness, p.Feature.InSharpness);
    set_i!(NVSDK_NGX_Parameter_Reset, p.InReset);
    set_f!(
        NVSDK_NGX_Parameter_MV_Scale_X,
        if p.InMVScaleX == 0.0 {
            1.0
        } else {
            p.InMVScaleX
        }
    );
    set_f!(
        NVSDK_NGX_Parameter_MV_Scale_Y,
        if p.InMVScaleY == 0.0 {
            1.0
        } else {
            p.InMVScaleY
        }
    );
    set_ptr!(NVSDK_NGX_Parameter_TransparencyMask, p.pInTransparencyMask);
    set_ptr!(NVSDK_NGX_Parameter_ExposureTexture, p.pInExposureTexture);
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_Mask,
        p.pInBiasCurrentColorMask
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Albedo,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ALBEDO as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Roughness,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ROUGHNESS as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Metallic,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_METALLIC as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Specular,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SPECULAR as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Subsurface,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SUBSURFACE as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Normals,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_NORMALS as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_ShadingModelId,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SHADINGMODELID as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_MaterialId,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_MATERIALID as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_8,
        p.GBufferSurface.pInAttrib[8]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_9,
        p.GBufferSurface.pInAttrib[9]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_10,
        p.GBufferSurface.pInAttrib[10]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_11,
        p.GBufferSurface.pInAttrib[11]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_12,
        p.GBufferSurface.pInAttrib[12]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_13,
        p.GBufferSurface.pInAttrib[13]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_14,
        p.GBufferSurface.pInAttrib[14]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_15,
        p.GBufferSurface.pInAttrib[15]
    );
    set_ui!(
        NVSDK_NGX_Parameter_TonemapperType,
        p.InToneMapperType as u32
    );
    set_ptr!(NVSDK_NGX_Parameter_MotionVectors3D, p.pInMotionVectors3D);
    set_ptr!(NVSDK_NGX_Parameter_IsParticleMask, p.pInIsParticleMask);
    set_ptr!(
        NVSDK_NGX_Parameter_AnimatedTextureMask,
        p.pInAnimatedTextureMask
    );
    set_ptr!(NVSDK_NGX_Parameter_DepthHighRes, p.pInDepthHighRes);
    set_ptr!(
        NVSDK_NGX_Parameter_Position_ViewSpace,
        p.pInPositionViewSpace
    );
    set_f!(
        NVSDK_NGX_Parameter_FrameTimeDeltaInMsec,
        p.InFrameTimeDeltaInMsec
    );
    set_ptr!(
        NVSDK_NGX_Parameter_RayTracingHitDistance,
        p.pInRayTracingHitDistance
    );
    set_ptr!(
        NVSDK_NGX_Parameter_MotionVectorsReflection,
        p.pInMotionVectorsReflections
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_X,
        p.InColorSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_Y,
        p.InColorSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_X,
        p.InDepthSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_Y,
        p.InDepthSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_X,
        p.InMVSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_Y,
        p.InMVSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_X,
        p.InTranslucencySubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_Y,
        p.InTranslucencySubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_X,
        p.InBiasCurrentColorSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_Y,
        p.InBiasCurrentColorSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_X,
        p.InOutputSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_Y,
        p.InOutputSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Width,
        p.InRenderSubrectDimensions.Width
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Height,
        p.InRenderSubrectDimensions.Height
    );
    set_f!(
        NVSDK_NGX_Parameter_DLSS_Pre_Exposure,
        if p.InPreExposure == 0.0 {
            1.0
        } else {
            p.InPreExposure
        }
    );
    set_f!(
        NVSDK_NGX_Parameter_DLSS_Exposure_Scale,
        if p.InExposureScale == 0.0 {
            1.0
        } else {
            p.InExposureScale
        }
    );
    set_i!(
        NVSDK_NGX_Parameter_DLSS_Indicator_Invert_X_Axis,
        p.InIndicatorInvertXAxis
    );
    set_i!(
        NVSDK_NGX_Parameter_DLSS_Indicator_Invert_Y_Axis,
        p.InIndicatorInvertYAxis
    );

    NVSDK_NGX_D3D12_EvaluateFeature_C(in_cmd_list, p_in_handle, p_in_params, None)
}

/// Equivalent of `NGX_D3D12_CREATE_DLSSD_EXT` from `nvsdk_ngx_helpers_dlssd_d3d.h`.
///
/// # Safety
///
/// All pointer parameters must be valid.
pub unsafe fn d3d12_create_dlssd_ext(
    in_cmd_list: *mut ID3D12GraphicsCommandList,
    in_creation_node_mask: u32,
    in_visibility_node_mask: u32,
    pp_out_handle: *mut *mut NVSDK_NGX_Handle,
    p_in_params: *mut NVSDK_NGX_Parameter,
    p_in_dlssd_create_params: *mut NVSDK_NGX_DLSSD_Create_Params,
) -> NVSDK_NGX_Result {
    let params = &*p_in_dlssd_create_params;

    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_CreationNodeMask.as_ptr().cast(),
        in_creation_node_mask,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_VisibilityNodeMask.as_ptr().cast(),
        in_visibility_node_mask,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_Width.as_ptr().cast(),
        params.InWidth,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_Height.as_ptr().cast(),
        params.InHeight,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_OutWidth.as_ptr().cast(),
        params.InTargetWidth,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_OutHeight.as_ptr().cast(),
        params.InTargetHeight,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_PerfQualityValue.as_ptr().cast(),
        params.InPerfQualityValue as i32,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Feature_Create_Flags
            .as_ptr()
            .cast(),
        params.InFeatureCreateFlags as i32,
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Enable_Output_Subrects
            .as_ptr()
            .cast(),
        if params.InEnableOutputSubrects { 1 } else { 0 },
    );
    NVSDK_NGX_Parameter_SetI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Denoise_Mode.as_ptr().cast(),
        NVSDK_NGX_DLSS_Denoise_Mode::NVSDK_NGX_DLSS_Denoise_Mode_DLUnified as i32,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_DLSS_Roughness_Mode.as_ptr().cast(),
        params.InRoughnessMode as u32,
    );
    NVSDK_NGX_Parameter_SetUI(
        p_in_params,
        NVSDK_NGX_Parameter_Use_HW_Depth.as_ptr().cast(),
        params.InUseHWDepth as u32,
    );

    NVSDK_NGX_D3D12_CreateFeature(
        in_cmd_list,
        NVSDK_NGX_Feature::NVSDK_NGX_Feature_RayReconstruction,
        p_in_params,
        pp_out_handle,
    )
}

/// Equivalent of `NGX_D3D12_EVALUATE_DLSSD_EXT` from `nvsdk_ngx_helpers_dlssd_d3d.h`.
///
/// # Safety
///
/// All pointer parameters must be valid.
pub unsafe fn d3d12_evaluate_dlssd_ext(
    in_cmd_list: *mut ID3D12GraphicsCommandList,
    p_in_handle: *mut NVSDK_NGX_Handle,
    p_in_params: *mut NVSDK_NGX_Parameter,
    p_in_dlssd_eval_params: *mut NVSDK_NGX_D3D12_DLSSD_Eval_Params,
) -> NVSDK_NGX_Result {
    let p = &*p_in_dlssd_eval_params;

    macro_rules! set_ptr {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetVoidPointer(p_in_params, $name.as_ptr().cast(), $val as *mut _);
        };
    }
    macro_rules! set_f {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetF(p_in_params, $name.as_ptr().cast(), $val);
        };
    }
    macro_rules! set_ui {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetUI(p_in_params, $name.as_ptr().cast(), $val);
        };
    }
    macro_rules! set_i {
        ($name:ident, $val:expr) => {
            NVSDK_NGX_Parameter_SetI(p_in_params, $name.as_ptr().cast(), $val);
        };
    }

    set_ptr!(NVSDK_NGX_Parameter_Color, p.pInColor);
    set_ptr!(NVSDK_NGX_Parameter_Output, p.pInOutput);
    set_ptr!(NVSDK_NGX_Parameter_Depth, p.pInDepth);
    set_ptr!(NVSDK_NGX_Parameter_MotionVectors, p.pInMotionVectors);
    set_f!(NVSDK_NGX_Parameter_Jitter_Offset_X, p.InJitterOffsetX);
    set_f!(NVSDK_NGX_Parameter_Jitter_Offset_Y, p.InJitterOffsetY);
    set_i!(NVSDK_NGX_Parameter_Reset, p.InReset);
    set_f!(
        NVSDK_NGX_Parameter_MV_Scale_X,
        if p.InMVScaleX == 0.0 {
            1.0
        } else {
            p.InMVScaleX
        }
    );
    set_f!(
        NVSDK_NGX_Parameter_MV_Scale_Y,
        if p.InMVScaleY == 0.0 {
            1.0
        } else {
            p.InMVScaleY
        }
    );
    set_ptr!(NVSDK_NGX_Parameter_TransparencyMask, p.pInTransparencyMask);
    set_ptr!(NVSDK_NGX_Parameter_ExposureTexture, p.pInExposureTexture);
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_Mask,
        p.pInBiasCurrentColorMask
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Albedo,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ALBEDO as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Roughness,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ROUGHNESS as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Metallic,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_METALLIC as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Specular,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SPECULAR as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Subsurface,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SUBSURFACE as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Normals,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_NORMALS as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_ShadingModelId,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SHADINGMODELID as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_MaterialId,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_MATERIALID as usize]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_8,
        p.GBufferSurface.pInAttrib[8]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_9,
        p.GBufferSurface.pInAttrib[9]
    );
    // DLSSD uses GBuffer_SpecularMvec for attrib[10]
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_SpecularMvec,
        p.pInMotionVectorsReflections
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_11,
        p.GBufferSurface.pInAttrib[11]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_12,
        p.GBufferSurface.pInAttrib[12]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_13,
        p.GBufferSurface.pInAttrib[13]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_14,
        p.GBufferSurface.pInAttrib[14]
    );
    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Atrrib_15,
        p.GBufferSurface.pInAttrib[15]
    );
    set_ui!(
        NVSDK_NGX_Parameter_TonemapperType,
        p.InToneMapperType as u32
    );
    set_ptr!(NVSDK_NGX_Parameter_MotionVectors3D, p.pInMotionVectors3D);
    set_ptr!(NVSDK_NGX_Parameter_IsParticleMask, p.pInIsParticleMask);
    set_ptr!(
        NVSDK_NGX_Parameter_AnimatedTextureMask,
        p.pInAnimatedTextureMask
    );
    set_ptr!(NVSDK_NGX_Parameter_DepthHighRes, p.pInDepthHighRes);
    set_ptr!(
        NVSDK_NGX_Parameter_Position_ViewSpace,
        p.pInPositionViewSpace
    );
    set_f!(
        NVSDK_NGX_Parameter_FrameTimeDeltaInMsec,
        p.InFrameTimeDeltaInMsec
    );
    set_ptr!(
        NVSDK_NGX_Parameter_RayTracingHitDistance,
        p.pInRayTracingHitDistance
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_X,
        p.InColorSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_Y,
        p.InColorSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_X,
        p.InDepthSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_Y,
        p.InDepthSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_X,
        p.InMVSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_Y,
        p.InMVSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_X,
        p.InTranslucencySubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_Y,
        p.InTranslucencySubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_X,
        p.InBiasCurrentColorSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_Y,
        p.InBiasCurrentColorSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_X,
        p.InOutputSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_Y,
        p.InOutputSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Width,
        p.InRenderSubrectDimensions.Width
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Height,
        p.InRenderSubrectDimensions.Height
    );
    set_f!(
        NVSDK_NGX_Parameter_DLSS_Pre_Exposure,
        if p.InPreExposure == 0.0 {
            1.0
        } else {
            p.InPreExposure
        }
    );
    set_f!(
        NVSDK_NGX_Parameter_DLSS_Exposure_Scale,
        if p.InExposureScale == 0.0 {
            1.0
        } else {
            p.InExposureScale
        }
    );
    set_i!(
        NVSDK_NGX_Parameter_DLSS_Indicator_Invert_X_Axis,
        p.InIndicatorInvertXAxis
    );
    set_i!(
        NVSDK_NGX_Parameter_DLSS_Indicator_Invert_Y_Axis,
        p.InIndicatorInvertYAxis
    );

    set_ptr!(
        NVSDK_NGX_Parameter_GBuffer_Emissive,
        p.GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_EMISSIVE as usize]
    );

    set_ptr!(NVSDK_NGX_Parameter_DiffuseAlbedo, p.pInDiffuseAlbedo);
    set_ptr!(NVSDK_NGX_Parameter_SpecularAlbedo, p.pInSpecularAlbedo);
    set_ptr!(NVSDK_NGX_Parameter_GBuffer_Normals, p.pInNormals);
    set_ptr!(NVSDK_NGX_Parameter_GBuffer_Roughness, p.pInRoughness);
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_DiffuseAlbedo_Subrect_Base_X,
        p.InDiffuseAlbedoSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_DiffuseAlbedo_Subrect_Base_Y,
        p.InDiffuseAlbedoSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_SpecularAlbedo_Subrect_Base_X,
        p.InSpecularAlbedoSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_SpecularAlbedo_Subrect_Base_Y,
        p.InSpecularAlbedoSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Normals_Subrect_Base_X,
        p.InNormalsSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Normals_Subrect_Base_Y,
        p.InNormalsSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Roughness_Subrect_Base_X,
        p.InRoughnessSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_Input_Roughness_Subrect_Base_Y,
        p.InRoughnessSubrectBase.Y
    );

    set_ptr!(NVSDK_NGX_Parameter_DLSSD_Alpha, p.pInAlpha);
    set_ptr!(NVSDK_NGX_Parameter_DLSSD_OutputAlpha, p.pInOutputAlpha);
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ReflectedAlbedo,
        p.pInReflectedAlbedo
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeParticles,
        p.pInColorBeforeParticles
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterParticles,
        p.pInColorAfterParticles
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeTransparency,
        p.pInColorBeforeTransparency
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterTransparency,
        p.pInColorAfterTransparency
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeFog,
        p.pInColorBeforeFog
    );
    set_ptr!(NVSDK_NGX_Parameter_DLSSD_ColorAfterFog, p.pInColorAfterFog);
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceSubsurfaceScatteringGuide,
        p.pInScreenSpaceSubsurfaceScatteringGuide
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceSubsurfaceScattering,
        p.pInColorBeforeScreenSpaceSubsurfaceScattering
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceSubsurfaceScattering,
        p.pInColorAfterScreenSpaceSubsurfaceScattering
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceRefractionGuide,
        p.pInScreenSpaceRefractionGuide
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceRefraction,
        p.pInColorBeforeScreenSpaceRefraction
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceRefraction,
        p.pInColorAfterScreenSpaceRefraction
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_DepthOfFieldGuide,
        p.pInDepthOfFieldGuide
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeDepthOfField,
        p.pInColorBeforeDepthOfField
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterDepthOfField,
        p.pInColorAfterDepthOfField
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseHitDistance,
        p.pInDiffuseHitDistance
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_SpecularHitDistance,
        p.pInSpecularHitDistance
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirection,
        p.pInDiffuseRayDirection
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirection,
        p.pInSpecularRayDirection
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirectionHitDistance,
        p.pInDiffuseRayDirectionHitDistance
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirectionHitDistance,
        p.pInSpecularRayDirectionHitDistance
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_Alpha_Subrect_Base_X,
        p.InAlphaSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_Alpha_Subrect_Base_Y,
        p.InAlphaSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_OutputAlpha_Subrect_Base_X,
        p.InOutputAlphaSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_OutputAlpha_Subrect_Base_Y,
        p.InOutputAlphaSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ReflectedAlbedo_Subrect_Base_X,
        p.InReflectedAlbedoSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ReflectedAlbedo_Subrect_Base_Y,
        p.InReflectedAlbedoSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeParticles_Subrect_Base_X,
        p.InColorBeforeParticlesSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeParticles_Subrect_Base_Y,
        p.InColorBeforeParticlesSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterParticles_Subrect_Base_X,
        p.InColorAfterParticlesSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterParticles_Subrect_Base_Y,
        p.InColorAfterParticlesSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeTransparency_Subrect_Base_X,
        p.InColorBeforeTransparencySubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeTransparency_Subrect_Base_Y,
        p.InColorBeforeTransparencySubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterTransparency_Subrect_Base_X,
        p.InColorAfterTransparencySubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterTransparency_Subrect_Base_Y,
        p.InColorAfterTransparencySubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeFog_Subrect_Base_X,
        p.InColorBeforeFogSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeFog_Subrect_Base_Y,
        p.InColorBeforeFogSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterFog_Subrect_Base_X,
        p.InColorAfterFogSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterFog_Subrect_Base_Y,
        p.InColorAfterFogSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceSubsurfaceScatteringGuide_Subrect_Base_X,
        p.InScreenSpaceSubsurfaceScatteringGuideSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceSubsurfaceScatteringGuide_Subrect_Base_Y,
        p.InScreenSpaceSubsurfaceScatteringGuideSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceSubsurfaceScattering_Subrect_Base_X,
        p.InColorBeforeScreenSpaceSubsurfaceScatteringSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceSubsurfaceScattering_Subrect_Base_Y,
        p.InColorBeforeScreenSpaceSubsurfaceScatteringSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceSubsurfaceScattering_Subrect_Base_X,
        p.InColorAfterScreenSpaceSubsurfaceScatteringSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceSubsurfaceScattering_Subrect_Base_Y,
        p.InColorAfterScreenSpaceSubsurfaceScatteringSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceRefractionGuide_Subrect_Base_X,
        p.InScreenSpaceRefractionGuideSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ScreenSpaceRefractionGuide_Subrect_Base_Y,
        p.InScreenSpaceRefractionGuideSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceRefraction_Subrect_Base_X,
        p.InColorBeforeScreenSpaceRefractionSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeScreenSpaceRefraction_Subrect_Base_Y,
        p.InColorBeforeScreenSpaceRefractionSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceRefraction_Subrect_Base_X,
        p.InColorAfterScreenSpaceRefractionSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterScreenSpaceRefraction_Subrect_Base_Y,
        p.InColorAfterScreenSpaceRefractionSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DepthOfFieldGuide_Subrect_Base_X,
        p.InDepthOfFieldGuideSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DepthOfFieldGuide_Subrect_Base_Y,
        p.InDepthOfFieldGuideSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeDepthOfField_Subrect_Base_X,
        p.InColorBeforeDepthOfFieldSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorBeforeDepthOfField_Subrect_Base_Y,
        p.InColorBeforeDepthOfFieldSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterDepthOfField_Subrect_Base_X,
        p.InColorAfterDepthOfFieldSubtectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_ColorAfterDepthOfField_Subrect_Base_Y,
        p.InColorAfterDepthOfFieldSubtectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseHitDistance_Subrect_Base_X,
        p.InDiffuseHitDistanceSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseHitDistance_Subrect_Base_Y,
        p.InDiffuseHitDistanceSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularHitDistance_Subrect_Base_X,
        p.InSpecularHitDistanceSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularHitDistance_Subrect_Base_Y,
        p.InSpecularHitDistanceSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirection_Subrect_Base_X,
        p.InDiffuseRayDirectionSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirection_Subrect_Base_Y,
        p.InDiffuseRayDirectionSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirection_Subrect_Base_X,
        p.InSpecularRayDirectionSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirection_Subrect_Base_Y,
        p.InSpecularRayDirectionSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirectionHitDistance_Subrect_Base_X,
        p.InDiffuseRayDirectionHitDistanceSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_DiffuseRayDirectionHitDistance_Subrect_Base_Y,
        p.InDiffuseRayDirectionHitDistanceSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirectionHitDistance_Subrect_Base_X,
        p.InSpecularRayDirectionHitDistanceSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSSD_SpecularRayDirectionHitDistance_Subrect_Base_Y,
        p.InSpecularRayDirectionHitDistanceSubrectBase.Y
    );

    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_WORLD_TO_VIEW_MATRIX,
        p.pInWorldToViewMatrix
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_VIEW_TO_CLIP_MATRIX,
        p.pInViewToClipMatrix
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayer,
        p.pInTransparencyLayer
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerOpacity,
        p.pInTransparencyLayerOpacity
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerMvecs,
        p.pInTransparencyLayerMvecs
    );
    set_ptr!(
        NVSDK_NGX_Parameter_DLSS_DisocclusionMask,
        p.pInDisocclusionMask
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayer_Subrect_Base_X,
        p.InTransparencyLayerSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayer_Subrect_Base_Y,
        p.InTransparencyLayerSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerOpacity_Subrect_Base_X,
        p.InTransparencyLayerOpacitySubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerOpacity_Subrect_Base_Y,
        p.InTransparencyLayerOpacitySubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerMvecs_Subrect_Base_X,
        p.InTransparencyLayerMvecsSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_TransparencyLayerMvecs_Subrect_Base_Y,
        p.InTransparencyLayerMvecsSubrectBase.Y
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_DisocclusionMask_Subrect_Base_X,
        p.InDisocclusionMaskSubrectBase.X
    );
    set_ui!(
        NVSDK_NGX_Parameter_DLSS_DisocclusionMask_Subrect_Base_Y,
        p.InDisocclusionMaskSubrectBase.Y
    );

    NVSDK_NGX_D3D12_EvaluateFeature_C(in_cmd_list, p_in_handle, p_in_params, None)
}
