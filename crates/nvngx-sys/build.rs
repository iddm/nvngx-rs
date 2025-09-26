use std::{
    env,
    path::{Path, PathBuf},
};

const SOURCE_FILE_PATH: &str = "src/bindings.cpp";

fn vulkan_sdk_include_directory() -> Option<PathBuf> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_windows = target_os.as_str() == "windows";

    // Mostly on Windows, the Vulkan headers don't exist in a common location but can be found based
    // on VULKAN_SDK, set by the Vulkan SDK installer.
    match env::var("VULKAN_SDK") {
        Ok(v) => Some(PathBuf::from(v).join(
            // On the Windows SDK the `Include` directory is capitalized
            if is_windows { "Include" } else { "include" },
        )),
        // TODO: On Windows, perhaps this should be an error with a link to the SDK installation?
        Err(env::VarError::NotPresent) if is_windows => {
            // On Windows there's no common include directory like `/usr/include` where Vulkan headers can be found
            panic!("When targeting Windows, the VULKAN_SDK environment variable must be set")
        }
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(e)) => {
            panic!("VULKAN_SDK environment variable is not Unicode: {e:?}")
        }
    }
}

fn compile_helpers() {
    let mut build = cc::Build::new();
    build.file(SOURCE_FILE_PATH);
    if let Some(inc) = vulkan_sdk_include_directory() {
        build.include(inc);
    }
    build.cpp(true);
    // MSVC doesn't have this warning, apparently, but GCC and Clang do.
    if env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() != "windows" {
        build.flag("-Wno-missing-field-initializers");
    }
    build.compile("ngx_helpers");
}

fn main() {
    compile_helpers();

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    assert_eq!(
        target_arch, "x86_64",
        "No libraries available for architecture `{target_arch}`"
    );

    // Tell cargo to tell rustc to link to the libraries.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dlss_library_path = Path::new(match target_os.as_str() {
        "windows" => "DLSS/lib/Windows_x86_64",
        "linux" => "DLSS/lib/Linux_x86_64",
        x => panic!("No libraries available for OS `{x}`"),
    });

    // Make the path relative to the crate source, where the DLSS submodule exists
    let dlss_library_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(dlss_library_path);

    // First link our Rust project against the right version of nvsdk_ngx
    match target_os.as_str() {
        "windows" => {
            // TODO: Only one architecture is included (and for vs201x)
            let link_library_path = dlss_library_path.join("x64");
            let windows_mt_suffix = windows_mt_suffix();
            // TODO select debug and/or _iterator0/1 when /MTd or /MDd are set.
            let dbg_suffix = if true { "" } else { "_dbg" };
            println!("cargo:rustc-link-lib=nvsdk_ngx{windows_mt_suffix}{dbg_suffix}");
            println!("cargo:rustc-link-search={}", link_library_path.display());
        }
        "linux" => {
            // On Linux there is only one link-library
            println!("cargo:rustc-link-lib=nvsdk_ngx");
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-search={}", dlss_library_path.display());
        }
        x => todo!("No libraries for {x}"),
    }

    #[cfg(feature = "generate-bindings")]
    generate_bindings();
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    use std::env;
    const HEADER_FILE_PATH: &str = "src/bindings.h";

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={HEADER_FILE_PATH}");

    let msrv = bindgen::RustTarget::stable(71, 0).unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut bindings = bindgen::Builder::default()
        .rust_target(msrv)
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .clang_arg("-Wno-missing-field-initializers")
        // The input header we would like to generate
        // bindings for.
        .header(HEADER_FILE_PATH)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Types and functions defined by the SDK:
        .allowlist_item(r"(PFN_)?NVSDK_NGX_\w+")
        // Single exception for a function that doesn't adhere to the naming standard:
        .allowlist_function("GetNGXResultAsString")
        // Exportable symbols defined by our `bindings.cpp/h`, wrapping `static inline` helpers
        .allowlist_function(r"HELPERS_NGX_\w+")
        // Disallow DirectX and CUDA APIs, for which we do not yet provide/implement bindings
        .blocklist_item(r"\w+D3[Dd]1[12]\w+")
        .blocklist_type("PFN_NVSDK_NGX_ResourceReleaseCallback")
        .blocklist_item(r"\w+CUDA\w+")
        // Disallow default implementation for this type, as bindgen
        // does not generate the proper default values for the
        // inline-initialised members. We provide the `Default`
        // implementation manually.
        .no_default("NVSDK_NGX_DLSSG_Opt_Eval_Params")
        // Disallow all other dependencies, like those from libc or Vulkan.
        .allowlist_recursively(false)
        .impl_debug(true)
        .impl_partialeq(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate_inline_functions(true)
        .bitfield_enum("NVSDK_NGX_DLSS_Feature_Flags")
        .bitfield_enum("NVSDK_NGX_Feature_Support_Result")
        // .generate_cstr(true)
        .disable_name_namespacing()
        .disable_nested_struct_naming()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        });

    if let Some(inc) = vulkan_sdk_include_directory() {
        bindings = bindings.clang_arg(format!("-I{}", inc.display()))
    }

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    bindings
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn windows_mt_suffix() -> &'static str {
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
    if target_features.contains("crt-static") {
        "_s"
    } else {
        "_d"
    }
}
