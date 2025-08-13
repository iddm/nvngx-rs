#include "../DLSS/include/nvsdk_ngx_helpers.h"
#include "../DLSS/include/nvsdk_ngx_params_dlssd.h"
#include "../DLSS/include/nvsdk_ngx_helpers_dlssd.h"

// Supersampling

NVSDK_NGX_Result HELPERS_NGX_D3D12_CREATE_DLSS_EXT(
    ID3D12GraphicsCommandList *pInCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSS_Create_Params *pInDlssCreateParams);

NVSDK_NGX_Result HELPERS_NGX_D3D12_EVALUATE_DLSS_EXT(
    ID3D12GraphicsCommandList *pInCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_D3D12_DLSS_Eval_Params *pInDlssEvalParams);

// Ray Reconstruction

NVSDK_NGX_Result HELPERS_NGX_D3D12_CREATE_DLSSD_EXT(
    ID3D12GraphicsCommandList *pInCmdList,
    unsigned int InCreationNodeMask,
    unsigned int InVisibilityNodeMask,
    NVSDK_NGX_Handle **ppOutHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_DLSSD_Create_Params *pInDlssDCreateParams);

NVSDK_NGX_Result HELPERS_NGX_D3D12_EVALUATE_DLSSD_EXT( 
    ID3D12GraphicsCommandList *pInCmdList,
    NVSDK_NGX_Handle *pInHandle,
    NVSDK_NGX_Parameter *pInParams,
    NVSDK_NGX_D3D12_DLSSD_Eval_Params *pInDlssDEvalParams);
