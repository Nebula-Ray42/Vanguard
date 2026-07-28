use ash::{Device, vk};
use std::ffi::CString;

use crate::scene::mesh::{get_attribute_descriptions, get_vertex_binding_description};
use render_api::engine_error::EngineError;

// =====================================================================
// Layer 3: Pipeline Builder (パイプライン生成の複雑さを隠蔽するヘルパー)
// =====================================================================
/// グラフィックスパイプラインを段階的に構築するためのビルダー。
pub struct PipelineBuilder<'a> {
    shader_stages: Vec<vk::PipelineShaderStageCreateInfo<'a>>,
    vertex_input_info: vk::PipelineVertexInputStateCreateInfo<'a>,
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    rasterizer: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisampling: vk::PipelineMultisampleStateCreateInfo<'a>,
    color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    depth_stencil: vk::PipelineDepthStencilStateCreateInfo<'a>,
    pipeline_layout: vk::PipelineLayout,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            shader_stages: Vec::new(),
            vertex_input_info: vk::PipelineVertexInputStateCreateInfo::default(),
            input_assembly: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false),
            viewport_state: vk::PipelineViewportStateCreateInfo::default(),
            rasterizer: vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE) // 両面描画
                .front_face(vk::FrontFace::CLOCKWISE) // 元の設定
                .depth_bias_enable(false),
            multisampling: vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1),
            color_blend_attachment: vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A,
                )
                .blend_enable(false),
            depth_stencil: vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false),
            pipeline_layout: vk::PipelineLayout::null(),
        }
    }

    pub fn with_shaders(mut self, stages: Vec<vk::PipelineShaderStageCreateInfo<'a>>) -> Self {
        self.shader_stages = stages;
        self
    }

    pub fn with_vertex_input(mut self, info: vk::PipelineVertexInputStateCreateInfo<'a>) -> Self {
        self.vertex_input_info = info;
        self
    }

    pub fn with_viewport_state(mut self, info: vk::PipelineViewportStateCreateInfo<'a>) -> Self {
        self.viewport_state = info;
        self
    }

    pub fn with_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.pipeline_layout = layout;
        self
    }

    /// 設定された情報をもとにパイプラインを生成します。
    ///
    /// # Errors
    /// レイアウトが設定されていない場合、またはVulkanによるパイプライン生成に失敗した場合に `EngineError` を返します。
    pub fn build(
        self,
        device: &Device,
        render_pass: vk::RenderPass,
    ) -> Result<vk::Pipeline, EngineError> {
        if self.pipeline_layout == vk::PipelineLayout::null() {
            return Err(EngineError::Legacy(
                "Pipeline Layout が設定されていません".to_string(),
            ));
        }

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&self.color_blend_attachment));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&self.shader_stages)
            .vertex_input_state(&self.vertex_input_info)
            .input_assembly_state(&self.input_assembly)
            .viewport_state(&self.viewport_state)
            .rasterization_state(&self.rasterizer)
            .multisample_state(&self.multisampling)
            .depth_stencil_state(&self.depth_stencil)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(self.pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        // FFI呼び出しのみをunsafeに。create_graphics_pipelinesのエラーは (Vec<Pipeline>, Result) を返すため .1 を参照する。
        unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| EngineError::Legacy(format!("GraphicsPipeline生成失敗: {:?}", e.1)))
                .map(|pipelines| pipelines[0])
        }
    }
}

// =====================================================================
// メインのパイプライン構造体
// =====================================================================
/// 描画状態の設定（シェーダー、深度テスト、ブレンド等）を保持するコンテキスト (Layer 3)
pub struct GraphicsPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

/// シェーダーバイナリをロードし、Vulkanのシェーダーモジュールを生成します。
///
/// # Errors
/// バイナリのパース失敗、またはモジュールの生成に失敗した場合に `EngineError` を返します。
fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule, EngineError> {
    let mut cursor = std::io::Cursor::new(code);

    let decoded = ash::util::read_spv(&mut cursor).map_err(|e| {
        EngineError::Legacy(format!("シェーダーコードのパースに失敗しました: {:?}", e))
    })?;

    let create_info = vk::ShaderModuleCreateInfo::default().code(&decoded);

    // FFI呼び出しのみを unsafe ブロックに隔離
    unsafe {
        device
            .create_shader_module(&create_info, None)
            .map_err(|e| {
                EngineError::Legacy(format!("シェーダーモジュールの生成に失敗しました: {:?}", e))
            })
    }
}

impl GraphicsPipeline {
    /// デフォルトのシェーダーを使用してグラフィックスパイプラインを初期化します。
    pub fn new(
        device: &Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<Self, EngineError> {
        // シェーダーの読み込み
        let vert_spv = include_bytes!("../../../../assets/shaders/main_vert.spv");
        let frag_spv = include_bytes!("../../../../assets/shaders/main_frag.spv");

        let vert_module = create_shader_module(device, vert_spv)?;
        let frag_module = create_shader_module(device, frag_spv)?;

        // CString::new はハードコードされた文字列("main")のため、失敗しないことが保証されている
        let main_name =
            CString::new("main").expect("C文字列の変換に失敗(Null文字が含まれています)");

        let shader_stages = vec![
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(&main_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(&main_name),
        ];

        // 頂点入力の設定
        let binding_desc = [get_vertex_binding_description()];
        let attrib_desc = get_attribute_descriptions();
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_desc)
            .vertex_attribute_descriptions(&attrib_desc);

        // Viewport (ウィンドウサイズ) の設定
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }];
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        // パイプラインレイアウト(Push Constantsなど) の設定
        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(size_of::<[f32; 16]>() as u32)];

        let layout_info =
            vk::PipelineLayoutCreateInfo::default().push_constant_ranges(&push_constant_ranges);

        let layout = unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| EngineError::Legacy(format!("PipelineLayout生成失敗: {}", e)))?
        };

        // ==========================================================
        // Builderを使ってパイプラインを構築
        // ==========================================================
        let pipeline = PipelineBuilder::new()
            .with_shaders(shader_stages)
            .with_vertex_input(vertex_input)
            .with_viewport_state(viewport_state)
            .with_layout(layout)
            .build(device, render_pass)?;

        // 6. クリーンアップ
        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        Ok(Self { layout, pipeline })
    }

    /// パイプラインとレイアウトを破棄します。
    pub unsafe fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
