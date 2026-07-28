use crate::render_pass_error::RenderPassError;
use ash::vk;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwapchainError {
    #[error("サーフェスの機能(Capabilities)の取得に失敗しました: {0}")]
    QueryCapabilities(#[source] vk::Result),

    #[error("対応するフォーマットの取得に失敗しました: {0}")]
    QueryFormats(#[source] vk::Result),

    #[error("対応するプレゼンモードの取得に失敗しました: {0}")]
    QueryPresentModes(#[source] vk::Result),

    #[error("利用可能なフォーマットが1つも見つかりませんでした")]
    NoFormatsAvailable,

    #[error("Swapchain本体の生成に失敗しました: {0}")]
    CreateSwapchain(#[source] vk::Result),

    #[error("Swapchain画像の取得に失敗しました: {0}")]
    GetImages(#[source] vk::Result),

    // ※以下の3つは、各ドメインをリファクタリングするまでの「一時的な避難所（Stringの受け皿）」です
    #[error("ImageViewの生成に失敗しました: {0}")]
    CreateImageView(String),

    #[error("Depthリソースの生成に失敗しました: {0}")]
    CreateDepthResource(String),

    #[error("RenderPassの生成に失敗しました: {0}")]
    CreateRenderPass(#[source] RenderPassError),

    #[error("Framebufferの生成に失敗しました: {0}")]
    CreateFramebuffer(String),
}
