use std::env;
use std::process::Command;

fn compile_shader(
    source_path: &str,
    entry_point: &str,
    stage: &str,
    output_path: &str,
) -> Result<(), String> {
    let status = Command::new("slangc")
        .args([
            source_path,
            "-target",
            "spirv",
            "-profile",
            "glsl_450",
            "-entry",
            entry_point,
            "-stage",
            stage,
            "-o",
            output_path,
        ])
        .status()
        .map_err(|e| {
            format!(
                "slangcコマンドの呼び出しに失敗しました。パスを確認してください: {}",
                e
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "シェーダーのコンパイルに失敗しました。ファイル: {}, ステージ: {}",
            source_path, stage
        ))
    }
}

// 追加: FlatBuffersのコンパイル関数
fn compile_flatbuffers(schema_path: &str, output_dir: &str) -> Result<(), String> {
    let status = Command::new("flatc")
        .args([
            "--rust",
            "-o",
            output_dir,  // 出力先ディレクトリ
            schema_path, // fbsファイルのパス
        ])
        .status()
        .map_err(|e| {
            format!(
                "flatcコマンドの呼び出しに失敗しました。パスを確認してください: {}",
                e
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "スキーマのコンパイルに失敗しました。ファイル: {}",
            schema_path
        ))
    }
}

fn main() -> Result<(), String> {
    // CARGO_MANIFEST_DIR は build.rs が配置されたクレートのディレクトリ (例: crates/renderer_vulkan)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // --- シェーダーのコンパイル処理 ---
    let shader_dir = format!("{}/../../assets/shaders", manifest_dir);
    let source_file = format!("{}/main.slang", shader_dir);

    println!("cargo:rerun-if-changed={}", source_file);

    let vert_out = format!("{}/main_vert.spv", shader_dir);
    compile_shader(&source_file, "vertexMain", "vertex", &vert_out)?;

    let frag_out = format!("{}/main_frag.spv", shader_dir);
    compile_shader(&source_file, "fragmentMain", "fragment", &frag_out)?;

    // --- FlatBuffersのコンパイル処理 (ここに追加) ---
    // ルート階層の schema ディレクトリを参照
    let schema_dir = format!("{}/../../schema", manifest_dir);
    let entity_schema = format!("{}/entity.fbs", schema_dir);

    // sharedクレート側に出力する (crates/shared/src を想定)
    // ※環境に合わせて出力先ディレクトリは調整してください
    let shared_out_dir = format!("{}/../shared/src", manifest_dir);

    // スキーマファイルが変更された時だけ再コンパイルを走らせる
    println!("cargo:rerun-if-changed={}", entity_schema);

    compile_flatbuffers(&entity_schema, &shared_out_dir)?;

    Ok(())
}
