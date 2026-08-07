find_package(Vulkan REQUIRED)
find_package(glfw3 CONFIG REQUIRED)
find_package(glm CONFIG REQUIRED)
find_package(flatbuffers CONFIG REQUIRED)
find_package(fastgltf CONFIG REQUIRED)

add_library(vanguard_renderer STATIC
        vk_backend/renderer_vulkan.cpp
        include/rey_renderer.h
        include/generated/render_command_generated.h
        vk_backend/commands/command_recorder.hpp
        include/engine_error.hpp
        vk_backend/core/sync_context.hpp
        vk_backend/pipeline/pipeline.hpp
        vk_backend/core/vulkan_context.hpp
        vk_backend/render/swapchain_target.hpp
        vk_backend/render/swapchain_target.cpp
        vk_backend/buffers/buffer.hpp
        vk_backend/buffers/buffer.cpp
        vk_backend/pipeline/pipeline.cpp
        scene/camera.hpp
        scene/mesh.hpp
        scene/transform.hpp
        include/render_types.hpp
        include/vk_mem_alloc.h
        include/vma_implementation.cpp
        vk_backend/descriptors/descriptor.hpp
        vk_backend/descriptors/descriptor.cpp
        vk_backend/commands/command_utils.hpp
        vk_backend/commands/command_utils.cpp
        vk_backend/render/vulkan_renderer.hpp
        include/glfw3.h
        include/glfw_includeI_vulkan.cpp
        "${SHADER_DIR}/main.slang"
        vk_backend/core/vulkan_context.cpp
        techniques/pbr/pbr_pipeline.hpp
        techniques/pbr/pbr_pipeline.cpp
        techniques/toon/toon_pipeline.hpp
        techniques/toon/toon_pipeline.cpp
        scene/mesh.cpp
        assets/image_loader.hpp
        assets/image_loader.cpp
        vk_backend/images/texture.hpp
        vk_backend/images/texture.cpp
        include/stb_image.h
        vk_backend/render/vulkan_renderer_core.cpp
        vk_backend/render/vulkan_renderer_init.cpp
        vk_backend/render/vulkan_renderer_draw.cpp
        scene/gltf_loader.cpp
        scene/gltf_loader.hpp
        vk_backend/buffers/vulkan_buffer_utils.hpp
        vk_backend/buffers/vulkan_buffer_utils.cpp
)

# このライブラリを使う人がインクルードすべきディレクトリを指定
target_include_directories(vanguard_renderer PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)

add_dependencies(vanguard_renderer generate_shaders)

# Vulkanなどの依存ライブラリをリンク
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
