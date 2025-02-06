use std::env;
use std::path::PathBuf;

const DLSS_LIBRARY_PATH: &str = "DLSS/lib/Linux_x86_64";
const HEADER_FILE_PATH: &str = "src/bindings.h";
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
    // This is the directory where the `c` library is located.
    let libdir_path = PathBuf::from("./")
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // panic!("outdir: {out_dir}");
    // This is the path to the intermediate object file for our library.
    let obj_path = libdir_path.join(format!("{out_dir}/ngx_helpers.o"));
    // This is the path to the static library file.
    let lib_path = libdir_path.join(format!("{out_dir}/libngx_helpers.a"));

    // Run `clang` to compile the source code file into an object file.
    let compile_job = std::process::Command::new("clang")
        .arg("-g")
        //.arg("-G0")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(libdir_path.join(SOURCE_FILE_PATH))
        .output()
        .expect("compile using `clang`");

    if !compile_job.status.success() {
        let stdout = String::from_utf8(compile_job.stdout).unwrap();
        let stderr = String::from_utf8(compile_job.stderr).unwrap();
        panic!("could not compile object file.\nStdout:\n{stdout}\n\nStderr:\n{stderr}");
    }

    // Run `ar` to generate the static library.
    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .arg(obj_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }

    // Link against the built helpers wrapper.
    println!(
        "cargo:rustc-link-search={}",
        libdir_path.join(out_dir).to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=ngx_helpers");
}

fn main() {
    compile_helpers();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", library_path());

    // Tell cargo to tell rustc to link to the libraries.
    println!("cargo:rustc-link-lib=nvsdk_ngx");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=dl");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={HEADER_FILE_PATH}");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(HEADER_FILE_PATH)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .impl_debug(true)
        .impl_partialeq(true)
        .prepend_enum_name(false)
        .generate_inline_functions(true)
        .bitfield_enum("NVSDK_NGX_DLSS_Feature_Flags")
        // .generate_cstr(true)
        // .bitfield_enum("NVSDK_NGX_DLSS_Feature_Flags")
        // .bitfield_enum("NVSDK_NGX_Result")
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
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
