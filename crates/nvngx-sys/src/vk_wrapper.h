#include <vulkan/vulkan.h>
#include "../DLSS/include/nvsdk_ngx_vk.h"
#include "../DLSS/include/nvsdk_ngx_helpers.h"
#include "../DLSS/include/nvsdk_ngx_helpers_vk.h"
#include "../DLSS/include/nvsdk_ngx_helpers_dlssd_vk.h"


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