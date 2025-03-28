use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod error;
pub mod types;

pub use error::Error;
pub use types::*;

#[async_trait]
pub trait Provider: Send + Sync + Debug {
    async fn send_message(&self, message: &str) -> Result<String, Error>;
    async fn set_model(&mut self, model: &str) -> Result<(), Error>;
    async fn list_models(&self) -> Result<Vec<String>, Error>;
    fn get_current_model(&self) -> String;
    async fn run_terminal(&mut self) -> Result<(), Error>;
    async fn get_loaded_models(&self) -> Result<Vec<String>, Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub name: String,
    pub agents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub description: String,
    pub team: Option<String>,
} 