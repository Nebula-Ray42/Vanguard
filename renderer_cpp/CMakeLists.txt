cmake_minimum_required(VERSION 3.20)
project(rey_renderer_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(SOURCE_FILES
        vulkan/renderer_vulkan.cpp
        vulkan/renderer_vulkan.cpp
        include/rey_renderer.h
        include/generated/render_command_generated.h
)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(rey_renderer STATIC ${SOURCE_FILES})

target_include_directories(rey_renderer PRIVATE
        include
        /opt/homebrew/include
)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|AppleClang|GNU")
    target_compile_options(rey_renderer PRIVATE -Wall -Wextra -Wpedantic)
endif()
