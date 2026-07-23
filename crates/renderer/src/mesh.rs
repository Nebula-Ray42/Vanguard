use ash::vk;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(size_of::<[f32; 3]>() as u32),
        ]
    }
}

pub const VERTICES: [Vertex; 36] = [
    // 前面 (赤)
    Vertex {
        pos: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    // 背面 (緑)
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    // 上面 (青)
    Vertex {
        pos: [-0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    // 下面 (黄)
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    // 右面 (マゼンタ)
    Vertex {
        pos: [0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    // 左面 (シアン)
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
];
