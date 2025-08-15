#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg(all(windows, feature = "dx"))]

use widestring::WideChar as wchar_t;
// Only import PODs (Plain Old Datastructures) in scope. The interface types
// from the Windows crate are ABI-incompatible because they already own the
// pointer inside (i.e. a Rust ID3D12Device is equivalent to a ComPtr<ID3D12Device> in Rust).
use windows::Win32::Graphics::Direct3D12::{D3D12_HEAP_PROPERTIES, D3D12_RESOURCE_DESC};

// helper struct for initialization. Should be ABI compatible
// https://learn.microsoft.com/en-us/windows/win32/direct3d12/cd3dx12-heap-properties
type CD3DX12_HEAP_PROPERTIES = D3D12_HEAP_PROPERTIES;

use super::ngx::*;
include!("dx_bindings.rs");
