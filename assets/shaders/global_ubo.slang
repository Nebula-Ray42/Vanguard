struct GlobalUbo {
    float4x4 viewProj;
    float3 cameraPos;
    float _padding;
};

// Set 0, Binding 0 にバインド
[[vk::binding(0, 0)]]
ConstantBuffer<GlobalUbo> globalUbo;
