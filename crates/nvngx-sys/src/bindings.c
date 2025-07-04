#include "bindings.h"

NVSDK_NGX_Resource_VK HELPERS_NVSDK_NGX_Create_ImageView_Resource_VK(
    VkImageView imageView,
    VkImage image,
    VkImageSubresourceRange subresourceRange,
    VkFormat format,
    unsigned int width,
    unsigned int height,
    bool readWrite) {

    return NVSDK_NGX_Create_ImageView_Resource_VK(
        imageView,
        image,
        subresourceRange,
        format,
        width,
        height,
        readWrite
    );
}

// Super-Sampling

NVSDK_NGX_Result HELPERS_NGX_DLSS_GET_OPTIMAL_SETTINGS(
    NVSDK_NGX_Parameter *pInParams,
    unsigned int InUserSelectedWidth,
    unsigned int InUserSelectedHeight,
    NVSDK_NGX_PerfQuality_Value InPerfQualityValue,
    unsigned int *pOutRenderOptimalWidth,
    unsigned int *pOutRenderOptimalHeight,
    unsigned int *pOutRenderMaxWidth,
    unsigned int *pOutRenderMaxHeight,
    unsigned int *pOutRenderMinWidth,
    unsigned int *pOutRenderMinHeight,
    float *pOutSharpness) {

    return NGX_DLSS_GET_OPTIMAL_SETTINGS(
        pInParams,
        InUserSelectedWidth,
        InUserSelectedHeight,
        InPerfQualityValue,
        pOutRenderOptimalWidth,
        pOutRenderOptimalHeight,
        pOutRenderMaxWidth,
        pOutRenderMaxHeight,
        pOutRenderMinWidth,
        pOutRenderMinHeight,
        pOutSharpness
    );
}

NVSDK_NGX_Result HELPERS_NGX_VULKAN_CREATE_DLSS_EXT1(
    VkDevice InDevice,
    VkCommandBuffer InCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSS_Create_Params *pInDlssCreateParams) {

    return NGX_VULKAN_CREATE_DLSS_EXT1(
        InDevice,
        InCmdList,
        InCreationNodeMask,
        InVisibilityNodeMask,
        ppOutHandle,
        pInParams,
        pInDlssCreateParams
    );
}

NVSDK_NGX_Result HELPERS_NGX_VULKAN_CREATE_DLSS_EXT(
    VkCommandBuffer InCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSS_Create_Params *pInDlssCreateParams) {

    return NGX_VULKAN_CREATE_DLSS_EXT(
        InCmdList,
        InCreationNodeMask,
        InVisibilityNodeMask,
        ppOutHandle,
        pInParams,
        pInDlssCreateParams
    );
}

NVSDK_NGX_Result HELPERS_NGX_VULKAN_EVALUATE_DLSS_EXT(
    VkCommandBuffer InCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_VK_DLSS_Eval_Params *pInDlssEvalParams) {

    // return NGX_VULKAN_EVALUATE_DLSS_EXT(
    //     InCmdList,
    //     pInHandle,
    //     pInParams,
    //     pInDlssEvalParams
    // );

    // FOR TESTING
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->Feature.pInColor);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInMotionVectors);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->Feature.pInOutput);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInDepth);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInTransparencyMask);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInExposureTexture);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInBiasCurrentColorMask);
	for (size_t i = 0; i <= 15; i++)
	{
        NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->GBufferSurface.pInAttrib[i]);
	}
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInMotionVectors3D);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInIsParticleMask);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInAnimatedTextureMask);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInDepthHighRes);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInPositionViewSpace);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInRayTracingHitDistance);
    NVSDK_NGX_ENSURE_VK_IMAGEVIEW(pInDlssEvalParams->pInMotionVectorsReflections);

    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_Color, pInDlssEvalParams->Feature.pInColor);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_Output, pInDlssEvalParams->Feature.pInOutput);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_Depth, pInDlssEvalParams->pInDepth);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_MotionVectors, pInDlssEvalParams->pInMotionVectors);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_Jitter_Offset_X, pInDlssEvalParams->InJitterOffsetX);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_Jitter_Offset_Y, pInDlssEvalParams->InJitterOffsetY);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_Sharpness, pInDlssEvalParams->Feature.InSharpness);
    NVSDK_NGX_Parameter_SetI(pInParams, NVSDK_NGX_Parameter_Reset, pInDlssEvalParams->InReset);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_MV_Scale_X, pInDlssEvalParams->InMVScaleX == 0.0f ? 1.0f : pInDlssEvalParams->InMVScaleX);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_MV_Scale_Y, pInDlssEvalParams->InMVScaleY == 0.0f ? 1.0f : pInDlssEvalParams->InMVScaleY);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_TransparencyMask, pInDlssEvalParams->pInTransparencyMask);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_ExposureTexture, pInDlssEvalParams->pInExposureTexture);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_Mask, pInDlssEvalParams->pInBiasCurrentColorMask);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Albedo, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ALBEDO]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Roughness, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_ROUGHNESS]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Metallic, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_METALLIC]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Specular, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SPECULAR]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Subsurface, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SUBSURFACE]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Normals, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_NORMALS]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_ShadingModelId, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_SHADINGMODELID]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_MaterialId, pInDlssEvalParams->GBufferSurface.pInAttrib[NVSDK_NGX_GBUFFER_MATERIALID]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_8, pInDlssEvalParams->GBufferSurface.pInAttrib[8]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_9, pInDlssEvalParams->GBufferSurface.pInAttrib[9]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_10, pInDlssEvalParams->GBufferSurface.pInAttrib[10]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_11, pInDlssEvalParams->GBufferSurface.pInAttrib[11]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_12, pInDlssEvalParams->GBufferSurface.pInAttrib[12]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_13, pInDlssEvalParams->GBufferSurface.pInAttrib[13]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_14, pInDlssEvalParams->GBufferSurface.pInAttrib[14]);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_GBuffer_Atrrib_15, pInDlssEvalParams->GBufferSurface.pInAttrib[15]);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_TonemapperType, pInDlssEvalParams->InToneMapperType);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_MotionVectors3D, pInDlssEvalParams->pInMotionVectors3D);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_IsParticleMask, pInDlssEvalParams->pInIsParticleMask);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_AnimatedTextureMask, pInDlssEvalParams->pInAnimatedTextureMask);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_DepthHighRes, pInDlssEvalParams->pInDepthHighRes);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_Position_ViewSpace, pInDlssEvalParams->pInPositionViewSpace);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_FrameTimeDeltaInMsec, pInDlssEvalParams->InFrameTimeDeltaInMsec);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_RayTracingHitDistance, pInDlssEvalParams->pInRayTracingHitDistance);
    NVSDK_NGX_Parameter_SetVoidPointer(pInParams, NVSDK_NGX_Parameter_MotionVectorsReflection, pInDlssEvalParams->pInMotionVectorsReflections);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_X, pInDlssEvalParams->InColorSubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Color_Subrect_Base_Y, pInDlssEvalParams->InColorSubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_X, pInDlssEvalParams->InDepthSubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Depth_Subrect_Base_Y, pInDlssEvalParams->InDepthSubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_X, pInDlssEvalParams->InMVSubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_MV_SubrectBase_Y, pInDlssEvalParams->InMVSubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_X, pInDlssEvalParams->InTranslucencySubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Translucency_SubrectBase_Y, pInDlssEvalParams->InTranslucencySubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_X, pInDlssEvalParams->InBiasCurrentColorSubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Input_Bias_Current_Color_SubrectBase_Y, pInDlssEvalParams->InBiasCurrentColorSubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_X, pInDlssEvalParams->InOutputSubrectBase.X);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Output_Subrect_Base_Y, pInDlssEvalParams->InOutputSubrectBase.Y);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Width , pInDlssEvalParams->InRenderSubrectDimensions.Width);
    NVSDK_NGX_Parameter_SetUI(pInParams, NVSDK_NGX_Parameter_DLSS_Render_Subrect_Dimensions_Height, pInDlssEvalParams->InRenderSubrectDimensions.Height);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_DLSS_Pre_Exposure, pInDlssEvalParams->InPreExposure == 0.0f ? 1.0f : pInDlssEvalParams->InPreExposure);
    NVSDK_NGX_Parameter_SetF(pInParams, NVSDK_NGX_Parameter_DLSS_Exposure_Scale, pInDlssEvalParams->InExposureScale == 0.0f ? 1.0f : pInDlssEvalParams->InExposureScale);
    NVSDK_NGX_Parameter_SetI(pInParams, NVSDK_NGX_Parameter_DLSS_Indicator_Invert_X_Axis, pInDlssEvalParams->InIndicatorInvertXAxis);
    NVSDK_NGX_Parameter_SetI(pInParams, NVSDK_NGX_Parameter_DLSS_Indicator_Invert_Y_Axis, pInDlssEvalParams->InIndicatorInvertYAxis);

    int val = NVSDK_NGX_VULKAN_EvaluateFeature_C(InCmdList, pInHandle, pInParams, NULL);
    return val;
}

// Ray Reconstruction
NVSDK_NGX_Result HELPERS_NGX_VULKAN_CREATE_DLSSD_EXT1(
    VkDevice InDevice,
    VkCommandBuffer InCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSSD_Create_Params *pInDlssDCreateParams) {
    return NGX_VULKAN_CREATE_DLSSD_EXT1(
        InDevice,
        InCmdList,
        InCreationNodeMask,
        InVisibilityNodeMask,
        ppOutHandle,
        pInParams,
        pInDlssDCreateParams
    );
}

NVSDK_NGX_Result HELPERS_NGX_VULKAN_EVALUATE_DLSSD_EXT(
    VkCommandBuffer InCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_VK_DLSSD_Eval_Params *pInDlssDEvalParams) {
    return NGX_VULKAN_EVALUATE_DLSSD_EXT(
        InCmdList,
        pInHandle,
        pInParams,
        pInDlssDEvalParams
    );
}
