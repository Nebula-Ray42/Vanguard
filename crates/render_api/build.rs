use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);

    // crates/render_api から見て2つ上のディレクトリ (rey_engine) をプロジェクトルートとする
    let project_root = manifest_path.parent().unwrap().parent().unwrap();

    let cpp_dir = project_root.join("renderer_cpp");
    let schema_file = project_root.join("schemas/render_command.fbs");

    // 【追加】ここでファイルが本当に存在するかRust側でチェックし、無ければパニックさせる
    if !schema_file.exists() {
        panic!("Schema file not found exactly at: {:?}", schema_file);
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    // 1. FlatBuffers の Rust用生成
    let status_rs = Command::new("flatc")
        .args(&["--rust", "-o", &out_dir])
        .arg(&schema_file)
        .status()
        .expect("Failed to execute flatc for Rust");
    assert!(status_rs.success(), "flatc (Rust) execution failed");

    // 2. FlatBuffers の C++用生成
    let cpp_generated_dir = cpp_dir.join("include/generated");
    fs::create_dir_all(&cpp_generated_dir).unwrap();
    let status_cpp = Command::new("flatc")
        .args(&["--cpp", "-o", cpp_generated_dir.to_str().unwrap()])
        .arg(&schema_file)
        .status()
        .expect("Failed to execute flatc for C++");
    assert!(status_cpp.success(), "flatc (C++) execution failed");

    // 3. CMake ビルド (compile_commands.json を出力するように設定)
    let dst = cmake::Config::new(&cpp_dir)
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .cxxflag("-I/opt/homebrew/include")
        .build_target("rey_renderer")
        .build();

    let comp_db = dst.join("build/compile_commands.json");
    let dest_db = cpp_dir.join("compile_commands.json");
    if comp_db.exists() {
        fs::copy(comp_db, dest_db).expect("Failed to copy compile_commands.json");
    }

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=rey_renderer");
    println!("cargo:rustc-link-lib=dylib=c++");

    // 4. compile_commands.json をプロジェクトルートにコピー (IDE連携用)
    let compile_commands_src = dst.join("build/compile_commands.json");
    let compile_commands_dst = project_root.join("compile_commands.json");

    if compile_commands_src.exists() {
        fs::copy(compile_commands_src, compile_commands_dst).ok();
    }
}
