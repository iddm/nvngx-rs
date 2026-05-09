//! Compiles every `*.wgsl` file under `examples/` to a `.spv` file in
//! `OUT_DIR`. Examples can then `include_bytes!(concat!(env!("OUT_DIR"),
//! "/<stem>.spv"))` to embed the SPIR-V at compile time.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let examples_dir = manifest_dir.join("examples");

    if !examples_dir.is_dir() {
        return;
    }

    for entry in walk_wgsl(&examples_dir) {
        compile_wgsl(&entry, &out_dir);
        println!("cargo:rerun-if-changed={}", entry.display());
    }
}

fn walk_wgsl(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(read) = fs::read_dir(dir) else {
        return out;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_wgsl(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("wgsl") {
            out.push(path);
        }
    }
    out
}

fn compile_wgsl(src_path: &Path, out_dir: &Path) {
    let src = fs::read_to_string(src_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", src_path.display()));

    let module = match naga::front::wgsl::parse_str(&src) {
        Ok(m) => m,
        Err(e) => panic!(
            "WGSL parse error in {}:\n{}",
            src_path.display(),
            e.emit_to_string(&src)
        ),
    };

    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap_or_else(|e| panic!("WGSL validation failed in {}: {e:?}", src_path.display()));

    let options = naga::back::spv::Options {
        lang_version: (1, 3),
        flags: naga::back::spv::WriterFlags::empty(),
        ..Default::default()
    };

    let words = naga::back::spv::write_vec(&module, &info, &options, None)
        .unwrap_or_else(|e| panic!("SPIR-V emit failed for {}: {e:?}", src_path.display()));

    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();

    let stem = src_path.file_stem().unwrap().to_string_lossy();
    let dst = out_dir.join(format!("{stem}.spv"));
    fs::write(&dst, &bytes)
        .unwrap_or_else(|e| panic!("write {}: {e}", dst.display()));
}
