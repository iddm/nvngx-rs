#ifndef BINDINGS_H
#define BINDINGS_H

#include <vulkan/vulkan.h>
#include "../DLSS/include/nvsdk_ngx_vk.h"
#include "../DLSS/include/nvsdk_ngx_helpers.h"
#include "../DLSS/include/nvsdk_ngx_helpers_vk.h"
#include "../DLSS/include/nvsdk_ngx_helpers_dlssd_vk.h"

NVSDK_NGX_Resource_VK HELPERS_NVSDK_NGX_Create_ImageView_Resource_VK(
    VkImageView imageView,
    VkImage image,
    VkImageSubresourceRange subresourceRange,
    VkFormat format,
    unsigned int width,
    unsigned int height,
    bool readWrite);

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
    float *pOutSharpness);

// Super-Sampling

NVSDK_NGX_Result HELPERS_NGX_VULKAN_CREATE_DLSS_EXT1(
    VkDevice InDevice,
    VkCommandBuffer InCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSS_Create_Params *pInDlssCreateParams);

NVSDK_NGX_Result HELPERS_NGX_VULKAN_EVALUATE_DLSS_EXT(
    VkCommandBuffer InCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_VK_DLSS_Eval_Params *pInDlssEvalParams);

// Ray Reconstruction

NVSDK_NGX_Result HELPERS_NGX_VULKAN_CREATE_DLSSD_EXT1(
    VkDevice InDevice,
    VkCommandBuffer InCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSSD_Create_Params *pInDlssDCreateParams);

NVSDK_NGX_Result HELPERS_NGX_VULKAN_EVALUATE_DLSSD_EXT(
    VkCommandBuffer InCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_VK_DLSSD_Eval_Params *pInDlssDEvalParams);

#endif // BINDINGS_H
