add_executable(vanguard_engine_tests
        test_main.cpp
)

list(APPEND CMAKE_PREFIX_PATH "/opt/homebrew")

target_include_directories(vanguard_engine_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/include
)

find_package(glfw3 REQUIRED)
find_package(GTest CONFIG REQUIRED)

target_link_libraries(vanguard_engine_tests PRIVATE
        glfw
        Vulkan::Vulkan
        "-framework Cocoa"
        "-framework IOKit"
        "-framework CoreFoundation"
        GTest::gtest
        GTest::gtest_main
        vanguard_renderer
)

target_include_directories(vanguard_engine_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/renderer/include
        ${CMAKE_SOURCE_DIR}/renderer
        /opt/homebrew/include
)

target_compile_options(vanguard_engine_tests PRIVATE
        -Wno-nullability-extension
        -Wno-nullability-completeness
        -Wno-unused-parameter
)

add_custom_command(TARGET vanguard_engine_tests POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        "${CMAKE_SOURCE_DIR}/assets"
        "$<TARGET_FILE_DIR:vanguard_engine_tests>/assets"
)

add_test(NAME CoreLogicTest COMMAND vanguard_engine_tests)
