use rscterm_core::Provider;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod lmstudio;

pub use lmstudio::LMStudioProvider;

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
pub struct Message {
    pub role: String,
    pub content: String,
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