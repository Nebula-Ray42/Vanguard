use crate::swapchain::SwapchainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("スワップチェーン層で致命的なエラーが発生しました: {0}")]
    Swapchain(#[from] SwapchainError),

    // TODO 今後、他のドメインエラーもここに追加していく
    // #[error("描画パスの構築に失敗しました: {0}")]
    // Render(#[from] RenderError),

    // TODO 一時的な String の避難所を追加、後々全ドメインのエラーを Enum 化する
    #[error("レガシーエラー（未移行）: {0}")]
    Legacy(String),
}

impl From<String> for EngineError {
    fn from(err: String) -> Self {
        EngineError::Legacy(err)
    }
}
