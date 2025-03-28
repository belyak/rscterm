use rscterm_core::error::Result;
use rscterm_core::{Error, Provider};
use crate::Message;
use reqwest::Client;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::{LMStudioConfig, LMStudioRequest, LMStudioResponse};

#[derive(Debug)]
pub struct LMStudioProvider {
    client: Client,
    config: LMStudioConfig,
}

impl LMStudioProvider {
    pub fn new(url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            config: LMStudioConfig { url, model },
        }
    }

    #[cfg(test)]
    pub fn with_emulation(url: String, model: String) -> Self {
        Self::new(url, model)
    }
}

#[async_trait]
impl Provider for LMStudioProvider {
    async fn send_message(&self, message: &str) -> Result<String> {
        let request = LMStudioRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            model: self.config.model.clone(),
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;

        let response: LMStudioResponse = response
            .json()
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;

        Ok(response.choices[0].message.content.clone())
    }

    async fn set_model(&self, model: &str) -> Result<()> {
        let valid_models = vec![
            "gemma-3-12b-it",
            "gemma-2b-it",
            "mistral-7b",
            "llama-2-7b",
            "codellama-7b",
        ];

        if !valid_models.contains(&model) {
            return Err(Error::InvalidModel(model.to_string()));
        }

        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "gemma-3-12b-it".to_string(),
            "gemma-2b-it".to_string(),
            "mistral-7b".to_string(),
            "llama-2-7b".to_string(),
            "codellama-7b".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );
        assert_eq!(provider.config.url, "http://localhost:1234");
        assert_eq!(provider.config.model, "test-model");
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );
        let models = provider.list_models().await.unwrap();
        assert!(!models.is_empty());
        assert!(models.contains(&"gemma-3-12b-it".to_string()));
    }

    #[tokio::test]
    async fn test_set_model() {
        let provider = LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );
        assert!(provider.set_model("gemma-3-12b-it").await.is_ok());
        assert!(provider.set_model("invalid-model").await.is_err());
    }
} 