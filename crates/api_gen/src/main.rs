//! Regenerates `nvngx-sys/src/{core,vk,dx}_bindings.rs` from the DLSS SDK
//! headers.
//!
//! Run with `cargo run -p api_gen` after updating the DLSS submodule. Requires
//! the Vulkan SDK headers (via `VULKAN_SDK` on Windows, system include path on Linux).

use std::{
    env,
    path::{Path, PathBuf},
};

const CORE_HEADER_FILE_PATH: &str = "src/core_bindings.h";
const VK_HEADER_FILE_PATH: &str = "src/vk_bindings.h";
const DX_HEADER_FILE_PATH: &str = "src/dx_bindings.h";

fn vulkan_sdk_include_directory() -> Option<PathBuf> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    let is_windows = target_os.as_str() == "windows";

    match env::var("VULKAN_SDK") {
        Ok(v) => Some(PathBuf::from(v).join(if is_windows { "Include" } else { "include" })),
        Err(env::VarError::NotPresent) if is_windows => {
            panic!("When targeting Windows, the VULKAN_SDK environment variable must be set")
        }
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(e)) => {
            panic!("VULKAN_SDK environment variable is not Unicode: {e:?}")
        }
    }
}

/// Shared bindgen configuration (MSRV target, enum styles, derive options).
///
/// `allowlist_recursively(false)` is deliberate: NGX's headers transitively
/// pull in `<vulkan/vulkan.h>` and the D3D12 SDK, so the default recursive
/// allowlist would drag in thousands of unrelated Vulkan / D3D12 / CRT types.
/// We instead enumerate the NGX surface explicitly via regex `allowlist_item`
/// patterns in each `generate_*_bindings` function, and bring in third-party
/// types (Vulkan via `ash::vk`, D3D12 via the `windows` crate) on the
/// consumer side.
fn common_builder(msrv: bindgen::RustTarget) -> bindgen::Builder {
    bindgen::Builder::default()
        .rust_target(msrv)
        .allowlist_recursively(false)
        .impl_debug(true)
        .impl_partialeq(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .bitfield_enum("NVSDK_NGX_DLSS_Feature_Flags")
        .bitfield_enum("NVSDK_NGX_Feature_Support_Result")
        .disable_name_namespacing()
        .disable_nested_struct_naming()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
}

fn generate_core_bindings(nvngx_sys_dir: &Path, msrv: bindgen::RustTarget) {
    let header_path = nvngx_sys_dir.join(CORE_HEADER_FILE_PATH);
    let out_path = nvngx_sys_dir.join("src").join("core_bindings.rs");

    let bindings = common_builder(msrv)
        .header(header_path.to_str().expect("header path is not UTF-8"))
        // Core (API-agnostic) types and functions:
        .allowlist_item(r"(PFN_)?NVSDK_NGX_\w+")
        .allowlist_function("GetNGXResultAsString")
        // Blocklist graphics-API-specific items that are defined in generic files or leak in via
        // transitive includes — these belong in the DX/CUDA binding files instead.
        .blocklist_item(r"(PFN_)?NVSDK_NGX_(D3D1[12]|CUDA)_\w+")
        .blocklist_item(r"(PFN_)?NVSDK_NGX_Parameter_(Set|Get)D3d1[12]Resource")
        .blocklist_item(r"(PFN_)?NVSDK_NGX_ResourceReleaseCallback");

    bindings
        .generate()
        .expect("Unable to generate core bindings")
        .write_to_file(&out_path)
        .expect("Couldn't write core bindings!");

    println!("Wrote {}", out_path.display());
}

fn generate_vk_bindings(nvngx_sys_dir: &Path, msrv: bindgen::RustTarget) {
    let header_path = nvngx_sys_dir.join(VK_HEADER_FILE_PATH);
    let out_path = nvngx_sys_dir.join("src").join("vk_bindings.rs");

    let mut bindings = common_builder(msrv)
        .header(header_path.to_str().expect("header path is not UTF-8"))
        // Only Vulkan-specific SDK types and functions:
        .allowlist_item(r"NVSDK_NGX_VULKAN_\w+")
        .allowlist_type(r"NVSDK_NGX_\w*VK\w*");

    if let Some(inc) = vulkan_sdk_include_directory() {
        bindings = bindings.clang_arg(format!("-I{}", inc.display()));
    }

    bindings
        .generate()
        .expect("Unable to generate Vulkan bindings")
        .write_to_file(&out_path)
        .expect("Couldn't write Vulkan bindings!");

    println!("Wrote {}", out_path.display());
}

fn generate_dx_bindings(nvngx_sys_dir: &Path, msrv: bindgen::RustTarget) {
    let header_path = nvngx_sys_dir.join(DX_HEADER_FILE_PATH);
    let out_path = nvngx_sys_dir.join("src").join("dx_bindings.rs");

    let bindings = common_builder(msrv)
        .header(header_path.to_str().expect("header path is not UTF-8"))
        // DX12-specific SDK types and functions:
        .allowlist_item(r"(PFN_)?NVSDK_NGX_D3D12_\w+")
        // D3D12 resource parameter accessors:
        .allowlist_item(r"(PFN_)?NVSDK_NGX_Parameter_(Set|Get)D3d12Resource")
        // DX12 resource alloc/release callbacks:
        .allowlist_type("PFN_NVSDK_NGX_ResourceReleaseCallback");

    bindings
        .generate()
        .expect("Unable to generate DX12 bindings")
        .write_to_file(&out_path)
        .expect("Couldn't write DX12 bindings!");

    println!("Wrote {}", out_path.display());
}

fn main() {
    // Resolve `nvngx-sys` relative to this binary's manifest dir so the tool
    // works regardless of where `cargo run` is invoked from.
    let nvngx_sys_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("api_gen crate must live under crates/")
        .join("nvngx-sys");

    let msrv = bindgen::RustTarget::stable(70, 0).unwrap();

    generate_core_bindings(&nvngx_sys_dir, msrv);
    generate_vk_bindings(&nvngx_sys_dir, msrv);
    generate_dx_bindings(&nvngx_sys_dir, msrv);
}
