set(SCHEMA_GENERATED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/generated")
file(MAKE_DIRECTORY ${SCHEMA_GENERATED_DIR})

set(FBS_FILES
        "${CMAKE_CURRENT_SOURCE_DIR}/render_command.fbs"
)

find_program(FLATC_EXECUTABLE flatc REQUIRED)

# スキーマファイルからC++ヘッダーを生成するコマンド
# （Python用なども必要であれば --cpp の後に --python 等を足してください）
add_custom_command(
        OUTPUT "${SCHEMA_GENERATED_DIR}/render_command_generated.h"
        COMMAND ${FLATC_EXECUTABLE} --cpp -o "${SCHEMA_GENERATED_DIR}" ${FBS_FILES}
        DEPENDS ${FBS_FILES}
        COMMENT "Compiling FlatBuffers schemas (C++)..."
)

add_custom_target(generate_schemas ALL
        DEPENDS "${SCHEMA_GENERATED_DIR}/render_command_generated.h"
)

add_library(vanta_schemas INTERFACE)
target_include_directories(vanta_schemas INTERFACE "${SCHEMA_GENERATED_DIR}")
add_dependencies(vanta_schemas generate_schemas)
