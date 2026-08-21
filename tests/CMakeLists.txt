add_executable(vanguard_engine_tests
        test_main.cpp
)

find_package(glfw3 REQUIRED)
find_package(GTest CONFIG REQUIRED)

target_include_directories(vanguard_engine_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/include
        ${CMAKE_SOURCE_DIR}/engine/renderer/include
        ${CMAKE_SOURCE_DIR}/engine/renderer
)

target_link_libraries(vanguard_engine_tests PRIVATE
        glfw
        Vulkan::Vulkan
        GTest::gtest
        GTest::gtest_main
        vanguard_renderer
)

if(APPLE)
    list(APPEND CMAKE_PREFIX_PATH "/opt/homebrew")
    target_include_directories(vanguard_engine_tests PRIVATE /opt/homebrew/include)
    target_link_libraries(vanguard_engine_tests PRIVATE
            "-framework Cocoa"
            "-framework IOKit"
            "-framework CoreFoundation"
    )
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
    target_compile_options(vanguard_engine_tests PRIVATE
            -Wno-nullability-extension
            -Wno-nullability-completeness
            -Wno-unused-parameter
    )
endif()

add_custom_command(TARGET vanguard_engine_tests POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        "${CMAKE_SOURCE_DIR}/assets"
        "$<TARGET_FILE_DIR:vanguard_engine_tests>/assets"
)

add_test(NAME CoreLogicTest COMMAND vanguard_engine_tests)
