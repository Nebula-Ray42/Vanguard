#version 450

// CPUから送られてくる3Dの頂点データ
layout(location = 0) in vec3 inPosition; // vec2 から vec3 に変更！
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

// MVP行列（Model・View・Projectionを掛け合わせた4x4行列）を受け取る
layout(push_constant) uniform PushConstants {
    mat4 mvp;
} push;

void main() {
    // 頂点に行列を掛けて、3D空間から画面上の2D座標へ変換する
    gl_Position = push.mvp * vec4(inPosition, 1.0);
    fragColor = inColor;
}
