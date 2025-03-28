pub mod lmstudio;

pub use lmstudio::LMStudioProvider;
use serde::{Deserialize, Serialize};
use rscterm_core::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioConfig {
    pub url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioRequest {
    pub messages: Vec<Message>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioResponse {
    pub choices: Vec<Choice>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: Message,
} 