use std::path::PathBuf;

const DLSS_LIBRARY_PATH: &str = "DLSS/lib/Linux_x86_64";
const SOURCE_FILE_PATH: &str = "src/bindings.c";

fn library_path() -> String {
    // let path = match DLSS_LIBRARY_TYPE {
    //     DlssLibraryType::Development => "dev",
    //     DlssLibraryType::Release => "rel",
    // };
    // let path = format!("{DLSS_LIBRARY_PATH}/{path}/");
    let path = DLSS_LIBRARY_PATH.to_owned();
    let mut path = PathBuf::from(path)
        .canonicalize()
        .expect("cannot canonicalize path");

    if is_docs_rs_build() {
        path.push(std::env::var("OUT_DIR").unwrap());
        path
    } else {
        path
    }
    .to_str()
    .unwrap()
    .to_owned()
}

fn is_docs_rs_build() -> bool {
    std::env::var("DOCS_RS").is_ok()
}

fn compile_helpers() {
    cc::Build::new()
        .file(SOURCE_FILE_PATH)
        .compile("ngx_helpers");
}

fn main() {
    compile_helpers();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", library_path());

    // Tell cargo to tell rustc to link to the libraries.
    println!("cargo:rustc-link-lib=nvsdk_ngx");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=dl");

    #[cfg(feature = "generate-bindings")]
    generate_bindings();
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    use std::env;
    const HEADER_FILE_PATH: &str = "src/bindings.h";

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={HEADER_FILE_PATH}");

    let msrv = bindgen::RustTarget::stable(70, 0).unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .rust_target(msrv)
        // The input header we would like to generate
        // bindings for.
        .header(HEADER_FILE_PATH)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Types and functions defined by the SDK:
        .allowlist_item("NVSDK_NGX_\\w+")
        // Single exception for a function that doesn't adhere to the naming standard:
        .allowlist_function("GetNGXResultAsString")
        // Exportable symbols defined by our `bindings.c/h`, wrapping `static inline` helpers
        .allowlist_function("HELPERS_NGX_\\w+")
        // Platform-specific type provided by libc
        .blocklist_type("wchar_t")
        // Disable all Vulkan types which will be imported in-scope from the `ash` crate
        .blocklist_type("Vk\\w+")
        .blocklist_type("PFN_vk\\w+")
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
        })
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
