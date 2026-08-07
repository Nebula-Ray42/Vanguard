cmake_minimum_required(VERSION 3.20)
project(vanguard LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(Vulkan REQUIRED)
find_package(glfw3 CONFIG REQUIRED)
find_package(glm CONFIG REQUIRED)
find_package(flatbuffers CONFIG REQUIRED)
find_package(fastgltf CONFIG REQUIRED)


add_subdirectory(tools/slang_generator)
find_program(SLANGC_EXECUTABLE slangc REQUIRED)

set(SHADER_DIR "${CMAKE_CURRENT_SOURCE_DIR}/assets/shaders")
set(GENERATED_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/renderer/include/generated")

file(MAKE_DIRECTORY ${GENERATED_INCLUDE_DIR})

set(SLANG_SOURCES
        "${SHADER_DIR}/main.slang"
)

set(SPV_VERT "${SHADER_DIR}/main_vert.spv")
set(SPV_FRAG "${SHADER_DIR}/main_frag.spv")
set(CPP_REFLECTION_HEADER "${GENERATED_INCLUDE_DIR}/shader_reflection.h")


add_custom_command(
        OUTPUT ${SPV_VERT} ${SPV_FRAG} ${CPP_REFLECTION_HEADER}
        COMMAND ${SLANGC_EXECUTABLE} "${SHADER_DIR}/main.slang" -target spirv -profile glsl_460 -entry vertexMain -stage vertex -o ${SPV_VERT}
        COMMAND ${SLANGC_EXECUTABLE} "${SHADER_DIR}/main.slang" -target spirv -profile glsl_460 -entry fragmentMain -stage fragment -o ${SPV_FRAG}
        COMMAND rey_slang_generator "${SHADER_DIR}/main.slang" "${CPP_REFLECTION_HEADER}"
        DEPENDS rey_slang_generator ${SLANG_SOURCES}
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        "${CMAKE_SOURCE_DIR}/assets"
        "$<TARGET_FILE_DIR:rey_engine_tests>/assets"
        COMMENT "Copying assets directory to test output directory..."
)

add_custom_target(generate_shaders ALL
        DEPENDS ${SPV_VERT} ${SPV_FRAG} ${CPP_REFLECTION_HEADER}
)

set(SOURCE_FILES
        renderer/vulkan/renderer_vulkan.cpp
        renderer/include/rey_renderer.h
        renderer/include/generated/render_command_generated.h
        renderer/vulkan/commands/command_recorder.h
        renderer/include/engine_error.h
        renderer/vulkan/core/sync_context.h
        renderer/vulkan/pipeline/pipeline.h
        renderer/vulkan/core/vulkan_context.h
        renderer/vulkan/render/swapchain_target.h
        renderer/vulkan/render/swapchain_target.cpp
        renderer/vulkan/buffers/buffer.h
        renderer/vulkan/buffers/buffer.cpp
        renderer/vulkan/pipeline/pipeline.cpp
        renderer/scene/camera.h
        renderer/scene/mesh.h
        renderer/scene/transform.h
        renderer/include/render_types.h
        renderer/include/vk_mem_alloc.h
        renderer/include/vma_implementation.cpp
        renderer/vulkan/descriptors/descriptor.h
        renderer/vulkan/descriptors/descriptor.cpp
        renderer/vulkan/commands/command_utils.h
        renderer/vulkan/commands/command_utils.cpp
        renderer/vulkan/render/vulkan_renderer.h
        renderer/include/glfw3.h
        renderer/include/glfw_includeI_vulkan.cpp
        "${SHADER_DIR}/main.slang"
        tests/test_main.cpp
        renderer/vulkan/core/vulkan_context.cpp
        renderer/techniques/pbr/pbr_pipeline.h
        renderer/techniques/pbr/pbr_pipeline.cpp
        renderer/techniques/toon/toon_pipeline.h
        renderer/techniques/toon/toon_pipeline.cpp
        renderer/scene/mesh.cpp
        renderer/assets/image_loader.h
        renderer/assets/image_loader.cpp
        renderer/vulkan/images/texture.h
        renderer/vulkan/images/texture.cpp
        renderer/include/stb_image.h
        renderer/vulkan/render/vulkan_renderer_core.cpp
        renderer/vulkan/render/vulkan_renderer_init.cpp
        renderer/vulkan/render/vulkan_renderer_draw.cpp
        renderer/scene/gltf_loader.cpp
        renderer/scene/gltf_loader.hpp
        renderer/vulkan/buffers/vulkan_buffer_utils.h
        renderer/vulkan/buffers/vulkan_buffer_utils.cpp
)

add_library(rey_renderer STATIC ${SOURCE_FILES})
add_dependencies(rey_renderer generate_shaders)

target_include_directories(rey_renderer PUBLIC
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer"
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer/include"
        "${CMAKE_CURRENT_SOURCE_DIR}/renderer/vulkan"
)

target_include_directories(rey_renderer PRIVATE ${GENERATED_INCLUDE_DIR})

target_link_libraries(rey_renderer PUBLIC
        Vulkan::Vulkan
        glfw
        glm::glm
        flatbuffers::flatbuffers
        fastgltf::fastgltf
)

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
