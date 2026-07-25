use std::env;
use std::process::Command;

fn compile_shader(source_path: &str, entry_point: &str, stage: &str, output_path: &str) -> Result<(), String> {
    let status = Command::new("slangc")
        .args([
            source_path,
            "-target", "spirv",
            "-profile", "glsl_450",
            "-entry", entry_point,
            "-stage", stage,
            "-o", output_path,
        ])
        .status()
        .map_err(|e| format!("slangcコマンドの呼び出しに失敗しました。パスを確認してください: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("シェーダーのコンパイルに失敗しました。ファイル: {}, ステージ: {}", source_path, stage))
    }
}

fn main() -> Result<(), String> {
    // CARGO_MANIFEST_DIR は build.rs が配置されたクレートのディレクトリ (例: crates/renderer)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // クレートのディレクトリから相対パスでルートの assets/shaders へ繋ぐ
    let shader_dir = format!("{}/../../assets/shaders", manifest_dir);
    let source_file = format!("{}/main.slang", shader_dir);

    // Cargoへの指示: shaderディレクトリ内の変更を監視
    println!("cargo:rerun-if-changed={}", source_file);

    let vert_out = format!("{}/main_vert.spv", shader_dir);
    compile_shader(&source_file, "vertexMain", "vertex", &vert_out)?;

    let frag_out = format!("{}/main_frag.spv", shader_dir);
    compile_shader(&source_file, "fragmentMain", "fragment", &frag_out)?;

    Ok(())
}