#ifndef DX_BINDINGS_H
#define DX_BINDINGS_H

// Unlike Vulkan, DLSS only ships `_vk` / `_cuda` helper variants — the
// non-suffixed `nvsdk_ngx_helpers*.h` files are the DX12 default. CUDA-only
// items are filtered out by regex in api_gen.
#include "../DLSS/include/nvsdk_ngx.h"
#include "../DLSS/include/nvsdk_ngx_helpers.h"
#include "../DLSS/include/nvsdk_ngx_helpers_dlssd_d3d.h"

#endif // DX_BINDINGS_H
