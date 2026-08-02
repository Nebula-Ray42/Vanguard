#include <slang.h>
#include <slang-com-ptr.h>
#include <iostream>
#include <fstream>
#include <string>

#include "generator_core.h"

int main(int argc, char** argv) {
    // ---------------------------------------------------------
    // 1. コマンドライン引数の検証
    // ---------------------------------------------------------
    if (argc < 3) {
        std::cerr << "[Error] Usage: rey_slang_generator <input.slang> <output.h>\n";
        return 1;
    }

    const char* inputPath = argv[1];
    const char* outputPath = argv[2];

    // ---------------------------------------------------------
    // 2. Slangコンパイラの準備と実行（ボイラープレート）
    // ---------------------------------------------------------
    Slang::ComPtr<slang::IGlobalSession> globalSession;
    if (SLANG_FAILED(slang::createGlobalSession(globalSession.writeRef()))) {
        std::cerr << "[Error] Failed to create Slang global session.\n";
        return 1;
    }

    // 早期リターン時のメモリ解放忘れを防止するために ComPtr を使用
    Slang::ComPtr<slang::ICompileRequest> request;
    globalSession->createCompileRequest(request.writeRef());

    // 言語をSlangに指定して翻訳単位を追加
    int translationUnitIndex = request->addTranslationUnit(SLANG_SOURCE_LANGUAGE_SLANG, nullptr);
    request->addTranslationUnitSourceFile(translationUnitIndex, inputPath);

    int targetIndex = request->addCodeGenTarget(SLANG_SPIRV);
    request->setTargetProfile(targetIndex, globalSession->findProfile("sm_6_0"));

    // コンパイル実行
    const SlangResult compileRes = request->compile();
    if (SLANG_FAILED(compileRes)) {
        std::cerr << "[Slang Compile Error]\n" << request->getDiagnosticOutput() << "\n";
        return 1;
    }

    // リフレクションデータの取得
    auto* reflection = reinterpret_cast<slang::ShaderReflection*>(request->getReflection());
    if (reflection == nullptr) {
        std::cerr << "[Error] Failed to get shader reflection data.\n";
        return 1;
    }

    // ---------------------------------------------------------
    // 3. [自作パート] 抽出・生成パイプラインの実行 (最新版)
    // ---------------------------------------------------------

    // フェーズ1：データの抽出（失敗したらここで早期リターン）
    auto extracted_result = rey_engine::slang_generator::extract_resource_bindings(reflection);
    if (!extracted_result.has_value()) {
        std::cerr << "[Error] Extraction Failed: " << extracted_result.error() << "\n";
        return 1;
    }

    // フェーズ2：C++コードの生成（失敗したらここで早期リターン）
    auto generated_result = rey_engine::slang_generator::generate_cpp_bindings(extracted_result.value());
    if (!generated_result.has_value()) {
        std::cerr << "[Error] Generation Failed: " << generated_result.error() << "\n";
        return 1;
    }

    // ---------------------------------------------------------
    // 4. 結果のファイル書き出し
    // ---------------------------------------------------------
    std::ofstream outFile(outputPath);
    if (!outFile.is_open()) {
        std::cerr << "[Error] Failed to open output file: " << outputPath << "\n";
        return 1;
    }

    // 成功した文字列データ（.value()）だけを書き込む
    outFile << generated_result.value();
    outFile.close();

    std::cout << "[Success] Generated reflection header: " << outputPath << "\n";
    return 0;
}
