add_executable(rey_engine_tests
        test_main.cpp
)

list(APPEND CMAKE_PREFIX_PATH "/opt/homebrew")

target_include_directories(rey_engine_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/include
)

find_package(glfw3 REQUIRED)

target_link_libraries(rey_engine_tests PRIVATE
        rey_renderer
        glfw
        Vulkan::Vulkan
        "-framework Cocoa"
        "-framework IOKit"
        "-framework CoreFoundation"
)

target_include_directories(rey_engine_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/include
        ${CMAKE_SOURCE_DIR}/src
        /opt/homebrew/include
)

target_compile_options(rey_engine_tests PRIVATE
        -Wno-nullability-extension
        -Wno-nullability-completeness
        -Wno-unused-parameter
)

target_link_libraries(rey_engine_tests PRIVATE rey_renderer)

add_custom_command(TARGET rey_engine_tests POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        "${CMAKE_SOURCE_DIR}/assets"
        "$<TARGET_FILE_DIR:rey_engine_tests>/assets"
)

# CTest にこのテストを登録
add_test(NAME CoreLogicTest COMMAND rey_engine_tests)
