#pragma once
#include <cstdint>
#include <cstddef>

// C++のコンパイラが勝手に関数名を変える（マングリング）のを防ぐ
#ifdef __cplusplus
extern "C" {
#endif

void execute_render_command(const uint8_t* ptr, size_t len);

#ifdef __cplusplus
}
#endif
