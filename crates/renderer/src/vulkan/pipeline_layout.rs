use ash::vk;
use anyhow::{Result, Context};

// 先ほど定義した、送りたい個別データの構造体
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants {
    pub model: [f32; 16], // 4x4行列
}

// パイプラインの大元の設計図を保持する構造体
pub struct PipelineLayoutSetup {
    pub layout: vk::PipelineLayout,
}

impl PipelineLayoutSetup {
    /// UBOの設計図と、Push Constantsの情報をまとめて、大元の設計図を作る関数
    pub fn new(
        device: &ash::Device,
        descriptor_set_layouts: &[vk::DescriptorSetLayout], // 前のステップで作ったUBOの設計図
    ) -> Result<Self> {

        // --------------------------------------------------------
        // 1. 個別データ (Push Constants) のルールを定義
        // --------------------------------------------------------
        // どのシェーダーに、どれくらいのサイズのデータを送るかを設定します
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX) // 頂点シェーダーへ送る
            .offset(0)
            // mem::size_of を使うことで、構造体のサイズ変更にも自動で追従（Always-Valid）
            .size(size_of::<PushConstants>() as u32);

        let push_constant_ranges = [push_constant_range];

        // --------------------------------------------------------
        // 2. 大元の設計図 (Pipeline Layout) を組み立てる
        // --------------------------------------------------------
        // UBOの設計図と、Push Constantsのルールをガッチャンコします
        let layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts)   // UBOなどの設計図のリスト
            .push_constant_ranges(&push_constant_ranges); // 個別データのルール

        // Vulkanに作成してもらう
        let layout = unsafe {
            device.create_pipeline_layout(&layout_create_info, None)
                .context("大元の設計図 (Pipeline Layout) の作成に失敗しました")?
        };

        // 失敗なく完了したら返す
        Ok(Self { layout })
    }
}
