use ash::vk;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderPassError {
    #[error("RenderPassの生成に失敗しました: {0}")]
    CreateFailed(#[source] vk::Result),
}