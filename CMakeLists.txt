cmake_minimum_required(VERSION 3.20)
project(rey_renderer_cpp LANGUAGES CXX)

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/renderer_cpp")
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/renderer_cpp/include")
include_directories("${CMAKE_CURRENT_SOURCE_DIR}/renderer_cpp/renderer_vulkan")

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(SOURCE_FILES
        renderer_cpp/renderer_vulkan/renderer_vulkan.cpp
        renderer_cpp/include/rey_renderer.h
        renderer_cpp/include/generated/render_command_generated.h
        renderer_cpp/renderer_vulkan/render/command_recorder.h
        renderer_cpp/include/engine_error.h
        renderer_cpp/renderer_vulkan/pipeline/sync_context.h
        renderer_cpp/renderer_vulkan/pipeline/pipeline.h
        renderer_cpp/renderer_vulkan/pipeline/vulkan_context.h
        renderer_cpp/renderer_vulkan/render/swapchain_target.h
        renderer_cpp/renderer_vulkan/render/swapchain_target.cpp
        renderer_cpp/renderer_vulkan/buffers/buffer.h
        renderer_cpp/renderer_vulkan/buffers/buffer.cpp
        renderer_cpp/renderer_vulkan/pipeline/pipeline.cpp
        renderer_cpp/renderer_vulkan/scene/camera.h
        renderer_cpp/renderer_vulkan/scene/mesh.h
        renderer_cpp/renderer_vulkan/scene/transform.h
        renderer_cpp/include/render_types.h
        renderer_cpp/include/vk_mem_alloc.h
        renderer_cpp/include/vma_implementation.cpp
        renderer_cpp/renderer_vulkan/descriptors/descriptor.h
        renderer_cpp/renderer_vulkan/descriptors/descriptor.cpp
        renderer_cpp/renderer_vulkan/buffers/command_utils.h
        renderer_cpp/renderer_vulkan/buffers/command_utils.cpp
        renderer_cpp/renderer_vulkan/render/vulkan_renderer.h
        renderer_cpp/renderer_vulkan/render/vulkan_renderer.cpp
        renderer_cpp/include/glfw3.h
        renderer_cpp/include/glfw_includeI_vulkan.cpp
)

set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
add_library(rey_renderer STATIC ${SOURCE_FILES})

# 1. Vulkan SDK内のVMAをヘッダー専用ライブラリとして手動定義
add_library(VulkanMemoryAllocator INTERFACE)

if(DEFINED ENV{VULKAN_SDK})
    target_include_directories(VulkanMemoryAllocator INTERFACE $ENV{VULKAN_SDK}/include)
else()
    target_include_directories(VulkanMemoryAllocator INTERFACE /Users/sakakibarayuto/VulkanSDK/1.4.350.1/macOS/include)
endif()

# 別名を定義
add_library(GPUOpen::VulkanMemoryAllocator ALIAS VulkanMemoryAllocator)

# 2. 必要なパッケージの検索（※VulkanMemoryAllocatorのfind_packageは削除）
find_package(Vulkan REQUIRED)
find_package(glm CONFIG REQUIRED)

# 3. ターゲットへのリンク設定
target_link_libraries(rey_renderer PRIVATE GPUOpen::VulkanMemoryAllocator)
target_link_libraries(rey_renderer PRIVATE glm::glm)

# 4. インクルードパスの設定
target_include_directories(rey_renderer PRIVATE
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer_cpp/include"
        /opt/homebrew/include
        ${Vulkan_INCLUDE_DIRS}
)

target_link_libraries(rey_renderer PUBLIC Vulkan::Vulkan)

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
add_subdirectory(renderer_cpp/renderer_vulkan/test)
