// main.slang

// --------------------------------------------------------
// 1. データ構造の定義（GLSLの layout(location = ...) を集約）
// --------------------------------------------------------

// CPUから送られてくる3Dの頂点データ
struct VertexInput {
    float3 position : POSITION;
    float3 color    : COLOR;
};

// 頂点シェーダーからフラグメントシェーダーへ渡すデータ
struct VertexOutput {
    float4 position : SV_Position; // gl_Position に相当
    float3 color    : COLOR;       // fragColor に相当
};

// MVP行列（Model・View・Projection）
struct PushConstants {
    float4x4 mvp;
};

// VulkanのPush Constantとして受け取る宣言
[[vk::push_constant]]
ConstantBuffer<PushConstants> push;


// --------------------------------------------------------
// 2. 頂点シェーダー（副作用のない関数として定義）
// --------------------------------------------------------
[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;

    // 注意: 行列の掛け算は `*` ではなく `mul()` 関数を使用する
    output.position = mul(push.mvp, float4(input.position, 1.0));
    output.color = input.color;

    return output;
}


// --------------------------------------------------------
// 3. フラグメントシェーダー（純粋に色を返す関数）
// --------------------------------------------------------
[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
    // input.position.z には 0.0 (手前) 〜 1.0 (奥) の深度値が入っています。
    // これをRGBにそのまま代入して、グレーの濃淡として出力します。
    float depth = input.position.z;

    return float4(depth, depth, depth, 1.0);
}