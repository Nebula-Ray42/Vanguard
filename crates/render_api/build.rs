use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);

    let project_root = manifest_path.parent().unwrap().parent().unwrap();

    let cpp_dir = project_root.join("renderer_cpp");
    let schema_file = project_root.join("schemas/render_command.fbs");

    if !schema_file.exists() {
        panic!("Schema file not found exactly at: {:?}", schema_file);
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    let status_rs = Command::new("flatc")
        .args(&["--rust", "-o", &out_dir])
        .arg(&schema_file)
        .status()
        .expect("Failed to execute flatc for Rust");
    assert!(status_rs.success(), "flatc (Rust) execution failed");

    let cpp_generated_dir = cpp_dir.join("include/generated");
    fs::create_dir_all(&cpp_generated_dir).unwrap();
    let status_cpp = Command::new("flatc")
        .args(&["--cpp", "-o", cpp_generated_dir.to_str().unwrap()])
        .arg(&schema_file)
        .status()
        .expect("Failed to execute flatc for C++");
    assert!(status_cpp.success(), "flatc (C++) execution failed");

    let cmake_build_dir = project_root.join("cmake-build-debug");

    let dst = cmake::Config::new(&project_root)
        .out_dir(&cmake_build_dir)
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .cxxflag(format!("-I{}", cpp_dir.join("include").display()))
        .cxxflag("-I/opt/homebrew/include")
        .build_target("rey_renderer")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=rey_renderer");
    println!("cargo:rustc-link-lib=dylib=c++");
}
