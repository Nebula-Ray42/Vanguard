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

    // 約30度（0.5ラジアン）傾ける
    float angle = 0.5;
    float c = cos(angle);
    float s = sin(angle);

    float3 p = input.position;
    float3 n = input.normalDir;

    // 1. Y軸回転（新しい変数 p1, n1 に計算結果を入れる）
    float3 p1 = float3(p.x * c + p.z * s, p.y, -p.x * s + p.z * c);
    float3 n1 = float3(n.x * c + n.z * s, n.y, -n.x * s + n.z * c);

    // 2. X軸回転（さらに新しい変数 p2, n2 に入れる）
    float3 p2 = float3(p1.x, p1.y * c - p1.z * s, p1.y * s + p1.z * c);
    float3 n2 = float3(n1.x, n1.y * c - n1.z * s, n1.y * s + n1.z * c);

    // 3. 画面に収めるための補正（切り捨て回避）
    p2.x *= 0.5; // 画面からはみ出ないように50%縮小
    p2.y *= 0.5;
    p2.z = p2.z * 0.5 + 0.5; // Z値を -0.5〜0.5 から 0.25〜0.75 に移動させ、安全圏に入れる

    // 計算結果を出力
    output.position = float4(p2, 1.0);
    output.color = input.color;
    output.normalDir = n2;

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