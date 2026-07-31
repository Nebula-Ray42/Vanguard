cmake_minimum_required(VERSION 3.20)
project(rey_renderer_cpp LANGUAGES CXX)

# C++23の厳格な要求
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# GNU拡張などを無効化し、純粋な標準C++を強制
set(CMAKE_CXX_EXTENSIONS OFF)

# ビルド対象のソースファイル（今は仮のモックファイルを指定）
set(SOURCE_FILES
        vulkan/renderer_vulkan.cpp
        vulkan/renderer_vulkan.cpp
        include/rey_renderer.h
        include/generated/render_command_generated.h
)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Rustにリンクするための「静的ライブラリ」として定義
add_library(rey_renderer STATIC ${SOURCE_FILES})

# インクルードディレクトリの設定（Rust/cxx側からもヘッダが見えるようにする）
target_include_directories(rey_renderer PRIVATE
        include
        /opt/homebrew/include
)

# 警告レベルを最大化（Rust側の #![forbid(unsafe_code)] の精神をC++にも）
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|AppleClang|GNU")
    target_compile_options(rey_renderer PRIVATE -Wall -Wextra -Wpedantic)
endif()
