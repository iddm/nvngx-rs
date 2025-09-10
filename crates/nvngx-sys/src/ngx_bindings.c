#include "ngx_bindings.h"

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