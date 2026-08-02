// ==========================================
// 1. データ構造の定義
// ==========================================

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

// TODO 将来の拡張用
struct Transform {
    float3 position;
    float4 rotation;
    float3 scale;
}
struct Entity {
    uint32_t id;
    uint32_t mesh_id;
    Transform transform;
}

// ==========================================
// 2. 外部からのデータ受け入れ口
// ==========================================
struct GlobalUbo {
    float4x4 viewProj;
    float3 cameraPos;
    float _padding;
};

[[vk::binding(0, 0)]]
ConstantBuffer<GlobalUbo> globalData;


struct PushConstants {
    float4x4 model; // mvpから、個別の位置情報(model)のみに変更
};

[[vk::push_constant]]
ConstantBuffer<PushConstants> push;


// ==========================================
// 3. 処理ロジック
// ==========================================

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;

    float4 worldPos = mul(push.model, float4(input.position, 1.0));

    output.position = mul(globalData.viewProj, worldPos);

    output.color = input.color;
    // TODO 法線は回転（Model）の影響を受けるため、本当はここで回転を掛ける必要がありますが、
    // まずは画面に絵を出すことを優先してそのまま渡します。
    output.normalDir = input.normalDir;

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
    // 1. ベクトルの正規化（長さを1にする）
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
