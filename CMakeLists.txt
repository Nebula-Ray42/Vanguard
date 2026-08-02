cmake_minimum_required(VERSION 3.20)
project(rey_renderer_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ==========================================
# 1. 外部ライブラリとツールの読み込み
# ==========================================
set(GLFW_BUILD_DOCS OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)

add_subdirectory(third_party/glfw)
add_subdirectory(tools/slang_generator)

find_package(Vulkan REQUIRED)
find_program(SLANGC_EXECUTABLE slangc REQUIRED)

# ==========================================
# 2. パスと変数の設定
# ==========================================
set(SHADER_DIR "${CMAKE_CURRENT_SOURCE_DIR}/assets/shaders")
set(GENERATED_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/renderer/include/generated")

# 初回ビルド時のためにディレクトリを作成
file(MAKE_DIRECTORY ${GENERATED_INCLUDE_DIR})

set(SLANG_SOURCES
        "${SHADER_DIR}/main.slang"
)

# 出力ファイル群
set(SPV_VERT "${SHADER_DIR}/main_vert.spv")
set(SPV_FRAG "${SHADER_DIR}/main_frag.spv")
set(CPP_REFLECTION_HEADER "${GENERATED_INCLUDE_DIR}/shader_reflection.h")

# ==========================================
# 3. 統合パイプライン (SPIR-Vコンパイル & C++ヘッダー生成)
# ==========================================
add_custom_command(
        OUTPUT ${SPV_VERT} ${SPV_FRAG} ${CPP_REFLECTION_HEADER}

        # 1. 頂点シェーダー (SPIR-V)
        COMMAND ${SLANGC_EXECUTABLE} "${SHADER_DIR}/main.slang" -target spirv -profile glsl_460 -entry vertexMain -stage vertex -o ${SPV_VERT}

        # 2. フラグメントシェーダー (SPIR-V)
        COMMAND ${SLANGC_EXECUTABLE} "${SHADER_DIR}/main.slang" -target spirv -profile glsl_460 -entry fragmentMain -stage fragment -o ${SPV_FRAG}

        # 3. C++リフレクションヘッダー生成 (自作ジェネレーターを実行)
        COMMAND rey_slang_generator "${SHADER_DIR}/main.slang" "${CPP_REFLECTION_HEADER}"

        DEPENDS rey_slang_generator ${SLANG_SOURCES}
        COMMENT "Compiling Slang shaders and generating C++ reflection headers..."
)

add_custom_target(generate_shaders ALL
        DEPENDS ${SPV_VERT} ${SPV_FRAG} ${CPP_REFLECTION_HEADER}
)

# ==========================================
# 4. レンダラー本体の定義
# ==========================================
set(SOURCE_FILES
        renderer/vulkan/renderer_vulkan.cpp
        renderer/include/rey_renderer.h
        renderer/include/generated/render_command_generated.h
        renderer/vulkan/render/command_recorder.h
        renderer/include/engine_error.h
        renderer/vulkan/pipeline/sync_context.h
        renderer/vulkan/pipeline/pipeline.h
        renderer/vulkan/pipeline/vulkan_context.h
        renderer/vulkan/render/swapchain_target.h
        renderer/vulkan/render/swapchain_target.cpp
        renderer/vulkan/buffers/buffer.h
        renderer/vulkan/buffers/buffer.cpp
        renderer/vulkan/pipeline/pipeline.cpp
        renderer/vulkan/scene/camera.h
        renderer/vulkan/scene/mesh.h
        renderer/vulkan/scene/transform.h
        renderer/include/render_types.h
        renderer/include/vk_mem_alloc.h
        renderer/include/vma_implementation.cpp
        renderer/vulkan/descriptors/descriptor.h
        renderer/vulkan/descriptors/descriptor.cpp
        renderer/vulkan/buffers/command_utils.h
        renderer/vulkan/buffers/command_utils.cpp
        renderer/vulkan/render/vulkan_renderer.h
        renderer/vulkan/render/vulkan_renderer.cpp
        renderer/include/glfw3.h
        renderer/include/glfw_includeI_vulkan.cpp
        "${SHADER_DIR}/main.slang"
        tests/test_main.cpp
        renderer/vulkan/pipeline/vulkan_context.cpp
)

add_library(rey_renderer STATIC ${SOURCE_FILES})

# C++本体より先に、必ずシェーダーとヘッダーの生成を完了させる
add_dependencies(rey_renderer generate_shaders)

# ==========================================
# 5. インクルードパスとリンクの設定
# ==========================================
target_include_directories(rey_renderer PUBLIC
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer"
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer/include"
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer/vulkan"
        "${CMAKE_CURRENT_SOURCE_DIR}/third_party/glm"
        "${CMAKE_SOURCE_DIR}/third_party/flatbuffers/include"
)

# 生成されたヘッダーファイルのパスを追加
target_include_directories(rey_renderer PRIVATE ${GENERATED_INCLUDE_DIR})

target_link_libraries(rey_renderer PUBLIC
        Vulkan::Vulkan
        glfw
)

# ==========================================
# 6. コンパイルオプションとPCH
# ==========================================
target_precompile_headers(rey_renderer PRIVATE
        <vulkan/vulkan.h>
        <expected>
        <vector>
        <string>
        <optional>
        <span>
        <variant>
        <array>
        <cstdint>
)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
    target_compile_options(rey_renderer PRIVATE
            -Wall -Wextra -Wpedantic
            -Wconversion
            -Wshadow
            -Werror=return-type
            -Wno-missing-field-initializers
            -Wno-nullability-extension
            -Wno-nullability-completeness
            -Wno-unused-parameter
    )
endif()

enable_testing()
add_subdirectory(tests)
