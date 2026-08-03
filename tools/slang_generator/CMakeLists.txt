cmake_minimum_required(VERSION 3.20)
project(ReySlangGenerator_Tool)

add_executable(rey_slang_generator main.cpp
        reflect_struct.cpp
        reflect_binding.cpp
        codegen_cpp.cpp
        generator_core.h)

target_compile_features(rey_slang_generator PRIVATE cxx_std_23)

if(MSVC)
    target_compile_options(rey_slang_generator PRIVATE /W4)
else()
    target_compile_options(rey_slang_generator PRIVATE -Wall -Wextra)
endif()

set(SLANG_SDK_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../third_party/slang")

target_include_directories(rey_slang_generator PRIVATE "${SLANG_SDK_DIR}/include")

target_link_directories(rey_slang_generator PRIVATE "${SLANG_SDK_DIR}/lib")

target_link_libraries(rey_slang_generator PRIVATE slang)

set_target_properties(rey_slang_generator PROPERTIES
        BUILD_WITH_INSTALL_RPATH TRUE
        INSTALL_RPATH "${SLANG_SDK_DIR}/lib"
)
