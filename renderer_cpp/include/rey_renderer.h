#pragma once
#include <cstdint>
#include <cstddef>

#ifdef __cplusplus
extern "C" {
#endif

void execute_render_command(const uint8_t* ptr, size_t len);

#ifdef __cplusplus
}
#endif
