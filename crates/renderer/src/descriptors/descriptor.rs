use ash::vk;
use anyhow::{Result, Context};

// 接続に関する情報をまとめて保持する構造体
pub struct DescriptorSetup {
    pub layout: vk::DescriptorSetLayout, // 設計図
    pub pool: vk::DescriptorPool,        // 部品箱
    pub set: vk::DescriptorSet,          // 実際の接続ケーブル
}

impl DescriptorSetup {
    /// ディスクリプタの構築とバッファの接続をすべて行う関数
    pub fn new(
        device: &ash::Device,
        buffer_info: vk::DescriptorBufferInfo, // 接続したいバッファの情報
    ) -> Result<Self> {

        // --------------------------------------------------------
        // ステップ1: 設計図 (Layout) を作る
        // --------------------------------------------------------
        // Binding 0番にUBOを接続するというルールを定義します
        let layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER) // UBOを使います
            .descriptor_count(1) // 1つだけ接続します
            .stage_flags(vk::ShaderStageFlags::VERTEX); // 頂点シェーダーで使います

        let bindings = [layout_binding];
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

        // 設計図をVulkanに作成してもらう
        let layout = unsafe {
            device.create_descriptor_set_layout(&layout_create_info, None)
                .context("設計図 (DescriptorSetLayout) の作成に失敗しました")?
        };

        // --------------------------------------------------------
        // ステップ2: 部品箱 (Pool) を用意する
        // --------------------------------------------------------
        // どんな種類のケーブルを、いくつ作るか（今回はUBOを1つ）を箱に設定します
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)];

        let pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1); // 箱から作れるケーブルの最大数

        // 部品箱をVulkanに作成してもらう
        let pool = unsafe {
            device.create_descriptor_pool(&pool_create_info, None)
                .context("部品箱 (DescriptorPool) の作成に失敗しました")?
        };

        // --------------------------------------------------------
        // ステップ3: 接続ケーブル (Set) を作る
        // --------------------------------------------------------
        let layouts = [layout]; // 先ほど作った設計図を渡す
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

        // 部品箱から設計図通りのケーブルを作成する
        let sets = unsafe {
            device.allocate_descriptor_sets(&alloc_info)
                .context("接続ケーブル (DescriptorSet) の作成に失敗しました")?
        };
        let set = sets[0]; // 1つしか作っていないので0番目を取り出す

        // --------------------------------------------------------
        // ステップ4: ケーブルにバッファを繋ぐ (Write)
        // --------------------------------------------------------
        let buffer_infos = [buffer_info]; // 引数で受け取ったバッファの情報
        let write_descriptor_set = vk::WriteDescriptorSet::default()
            .dst_set(set) // 繋ぎ先はこのケーブル
            .dst_binding(0) // 0番の接続口に
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER) // UBOとして
            .buffer_info(&buffer_infos); // このバッファを繋ぐ

        let write_sets = [write_descriptor_set];

        unsafe {
            device.update_descriptor_sets(&write_sets, &[]);
        }

        Ok(Self { layout, pool, set })
    }
}
