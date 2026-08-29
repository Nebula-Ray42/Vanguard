find_package(Vulkan REQUIRED)
find_package(glfw3 CONFIG REQUIRED)
find_package(glm CONFIG REQUIRED)
find_package(flatbuffers CONFIG REQUIRED)
find_package(fastgltf CONFIG REQUIRED)

add_library(vanguard_renderer STATIC
        vk_backend/renderer_vulkan.cpp
        include/rey_renderer.h
        include/generated/render_command_generated.h
        vk_backend/commands/command_recorder.h
        include/engine_error.h
        vk_backend/core/sync_context.h
        vk_backend/pipeline/pipeline.h
        vk_backend/core/vulkan_context.h
        vk_backend/render/swapchain_target.h
        vk_backend/render/swapchain_target.cpp
        vk_backend/buffers/buffer.h
        vk_backend/buffers/buffer.cpp
        vk_backend/pipeline/pipeline.cpp
        scene/camera.h
        scene/mesh.h
        scene/transform.h
        include/render_types.h
        include/ext/vk_mem_alloc.h
        include/ext/vma_implementation.cpp
        vk_backend/descriptors/descriptor.h
        vk_backend/descriptors/descriptor.cpp
        vk_backend/commands/command_utils.h
        vk_backend/commands/command_utils.cpp
        vk_backend/render/vulkan_renderer.h
        include/ext/glfw3.h
        include/ext/glfw_includeI_vulkan.cpp
        "${SHADER_DIR}/main.slang"
        vk_backend/core/vulkan_context.cpp
        techniques/pbr/pbr_pipeline.hpp
        techniques/pbr/pbr_pipeline.cpp
        techniques/toon/toon_pipeline.hpp
        techniques/toon/toon_pipeline.cpp
        scene/mesh.cpp
        assets/image_loader.hpp
        assets/image_loader.cpp
        vk_backend/images/texture.h
        vk_backend/images/texture.cpp
        include/ext/stb_image.h
        vk_backend/render/vulkan_renderer_core.cpp
        vk_backend/render/vulkan_renderer_init.cpp
        vk_backend/render/vulkan_renderer_draw.cpp
        scene/gltf_loader.cpp
        scene/gltf_loader.h
        vk_backend/buffers/vulkan_buffer_utils.h
        vk_backend/buffers/vulkan_buffer_utils.cpp
        scene/camera.cpp
        vk_backend/frame_graph/render_graph_types.h
        vk_backend/frame_graph/graph_compiler.h
        vk_backend/frame_graph/graph_compiler.cpp
        vk_backend/frame_graph/graph_executor.h
        vk_backend/frame_graph/graph_executor.cpp
        vk_backend/frame_graph/graph_builder.h
)

target_include_directories(vanguard_renderer PUBLIC
        .
        include
)

add_dependencies(vanguard_renderer generate_shaders)

target_link_libraries(vanguard_renderer PRIVATE
        Vulkan::Vulkan
)

target_link_libraries(vanguard_renderer PUBLIC
        Vulkan::Vulkan
        glfw
        glm::glm
        flatbuffers::flatbuffers
        fastgltf::fastgltf
)

target_precompile_headers(vanguard_renderer PRIVATE
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
    target_compile_options(vanguard_renderer PRIVATE
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
