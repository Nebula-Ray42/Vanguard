#version 450

layout(location = 0) out vec3 out_color;

// 【追加】CPUから高速に送られてくる少量のデータを受け取るブロック
layout(push_constant) uniform PushConstants {
    vec2 offset; // X方向とY方向のズレ（f32 x 2）
} push_constants;

vec2 POSITIONS[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

vec3 COLORS[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
    // 【変更】元の頂点座標に、CPUから送られてきたオフセット（ズレ）を足し込む
    gl_Position = vec4(POSITIONS[gl_VertexIndex] + push_constants.offset, 0.0, 1.0);
    out_color = COLORS[gl_VertexIndex];
}
