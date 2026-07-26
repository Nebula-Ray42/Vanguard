// --------------------------------------------------------
// 1. データ構造の定義
// --------------------------------------------------------

// CPUから送られてくる3Dの頂点データ
struct VertexInput {
     [[vk::location(0)]] float3 position : POSITION;
     [[vk::location(1)]] float3 color : COLOR;
     [[vk::location(2)]] float3 normalDir : NORMAL;
 }

 // 頂点シェーダーからフラグメントシェーダーへ渡すデータ
 struct VertexOutput {
     float4 position : SV_Position;
     float3 color : COLOR;
     float3 normalDir : NORMAL;
 }

// MVP行列（Model・View・Projection）
struct PushConstants {
    float4x4 mvp;
};

// VulkanのPush Constantとして受け取る宣言
[[vk::push_constant]]
ConstantBuffer<PushConstants> push;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;

    // CPUから送られてきたMVP行列を使って頂点を正しく変換する
    output.position = mul(float4(input.position, 1.0), push.mvp);

    // 色と向き（法線）はそのまま後ろへ受け渡す
    output.color = input.color;
    output.normalDir = input.normalDir;

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
    // 1. ベクトルの正規化（長さを1にする）
    // 受け取った法線の名前を normalDir に変えたのでここも合わせます
    float3 N = normalize(input.normalDir);

    // 2. ライトの方向（仮に右上奥から手前下へ向かう光とする）
    float3 lightDir = normalize(float3(1.0, 1.0, 1.0));

    // 3. ランバート反射モデルの数式: I = max(0, N・L)
    float diffuse = max(0.0, dot(N, lightDir));

    // 4. 環境光（完全に真っ暗になるのを防ぐための下積み光）
    float ambient = 0.2;

    // 5. 最終的な光の強さ
    float lightIntensity = ambient + diffuse;

    // ベースカラーに光の強さを掛けて出力
    float3 finalColor = input.color * lightIntensity;

    return float4(finalColor, 1.0);
}