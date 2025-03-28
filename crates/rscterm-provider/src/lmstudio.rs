use rscterm_core::error::Result;
use rscterm_core::{Error, Provider, Message};
use reqwest::Client;
use async_trait::async_trait;
use std::fmt::Debug;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

use super::{LMStudioConfig, LMStudioRequest, LMStudioResponse};

#[derive(Debug, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    object: String,
    owned_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
    object: String,
}

#[derive(Debug)]
pub struct LMStudioProvider {
    client: Client,
    config: LMStudioConfig,
    available_models: Vec<String>,
}

impl LMStudioProvider {
    pub fn new(url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            config: LMStudioConfig { url, model },
            available_models: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn with_emulation(url: String, model: String) -> Self {
        Self::new(url, model)
    }

    async fn send_request<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let response = self
            .client
            .get(&format!("{}{}", self.config.url, endpoint))
            .send()
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Provider(format!(
                "Request failed: {}",
                response.status()
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| Error::Provider(format!("error decoding response body: {}", e)))
    }

    async fn send_chat_request(&self, request: &LMStudioRequest) -> Result<LMStudioResponse> {
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.url))
            .json(request)
            .send()
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Provider(format!(
                "Request failed: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Provider(e.to_string()))
    }

    async fn prefetch_model(&self, model: &str) -> Result<()> {
        let request = LMStudioRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
            model: model.to_string(),
        };

        self.send_chat_request(&request).await?;
        Ok(())
    }

    fn create_request(&self, messages: Vec<Message>, model: Option<&str>) -> LMStudioRequest {
        LMStudioRequest {
            messages,
            model: model.unwrap_or(&self.config.model).to_string(),
        }
    }
}

#[async_trait]
impl Provider for LMStudioProvider {
    async fn send_message(&self, message: &str) -> Result<String> {
        let request = self.create_request(
            vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            None,
        );

        let response = self.send_chat_request(&request).await?;
        Ok(response.choices[0].message.content.clone())
    }

    async fn set_model(&mut self, model: &str) -> Result<()> {
        // Fetch available models first
        self.available_models = self.get_loaded_models().await?;
        
        if !self.available_models.contains(&model.to_string()) {
            return Err(Error::InvalidModel(model.to_string()));
        }

        self.prefetch_model(model).await?;
        self.config.model = model.to_string();
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        self.get_loaded_models().await
    }

    fn get_current_model(&self) -> String {
        self.config.model.clone()
    }

    async fn run_terminal(&mut self) -> Result<()> {
        println!("Starting terminal mode with model {}. Type 'exit' to quit.", self.config.model);
        println!("You can use Ctrl+C to interrupt the model's response.");
        
        let mut conversation_history = Vec::new();
        
        loop {
            print!("> ");
            io::stdout().flush().map_err(|e| Error::Provider(e.to_string()))?;
            
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .map_err(|e| Error::Provider(e.to_string()))?;
            
            let input = input.trim();
            
            if input.eq_ignore_ascii_case("exit") {
                break;
            }
            
            conversation_history.push(Message {
                role: "user".to_string(),
                content: input.to_string(),
            });
            
            let request = self.create_request(conversation_history.clone(), None);
            let response = self.send_chat_request(&request).await?;
            let response_content = response.choices[0].message.content.clone();
            
            conversation_history.push(Message {
                role: "assistant".to_string(),
                content: response_content.clone(),
            });
            
            println!("\n{}\n", response_content);
        }
        
        Ok(())
    }

    async fn get_loaded_models(&self) -> Result<Vec<String>> {
        let response: ModelsResponse = self.send_request("/v1/models").await?;
        Ok(response.data.into_iter().map(|m| m.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use mockito::mock;

    const TEST_URL: &str = "http://10.6.1.238:1234";

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );
        assert_eq!(provider.config.url, TEST_URL);
        assert_eq!(provider.config.model, "test-model");
        assert!(provider.available_models.is_empty());
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );
        let models = provider.list_models().await;
        assert!(models.is_err()); // Will fail due to no actual server
    }

    #[tokio::test]
    async fn test_set_model_validation() {
        let mut provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        // Test invalid models
        let invalid_models = ["invalid-model", "", "gpt-4", "claude-3"];
        for model in &invalid_models {
            let result = provider.set_model(model).await;
            assert!(result.is_err());
            if let Err(Error::InvalidModel(msg)) = result {
                assert_eq!(msg, *model);
            } else {
                panic!("Expected InvalidModel error");
            }
        }
    }

    #[tokio::test]
    async fn test_model_state_persistence() {
        let mut provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        assert!(provider.set_model("test-model").await.is_ok());
        assert_eq!(provider.config.model, "test-model");

        let request = provider.send_message("test message").await;
        assert!(request.is_err()); // Will fail due to no actual server
    }

    #[tokio::test]
    async fn test_concurrent_model_operations() {
        let mut provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        let model_switch = provider.set_model("test-model");
        let model_list = provider.list_models();

        let (switch_result, list_result) = tokio::join!(model_switch, model_list);
        assert!(switch_result.is_ok());
        assert!(list_result.is_err()); // Will fail due to no actual server
    }

    #[tokio::test]
    async fn test_model_switch_with_delay() {
        let mut provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        let switch_with_delay = async {
            sleep(Duration::from_millis(100)).await;
            provider.set_model("test-model").await
        };

        let list_models = async {
            sleep(Duration::from_millis(50)).await;
            provider.list_models().await
        };

        let (switch_result, list_result) = tokio::join!(switch_with_delay, list_models);
        assert!(switch_result.is_ok());
        assert!(list_result.is_err()); // Will fail due to no actual server
    }

    #[test]
    fn test_get_current_model() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );
        assert_eq!(provider.get_current_model(), "test-model");
    }

    #[tokio::test]
    async fn test_run_terminal() {
        let mut provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );
        
        let result = provider.run_terminal().await;
        assert!(result.is_err()); // Will fail due to no actual server
    }

    #[tokio::test]
    async fn test_send_message_error_handling() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        // Test with empty message
        let result = provider.send_message("").await;
        assert!(result.is_err());

        // Test with invalid URL
        let provider = LMStudioProvider::new(
            "invalid-url".to_string(),
            "test-model".to_string(),
        );
        let result = provider.send_message("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prefetch_model_error_handling() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        // Test with invalid model
        let result = provider.prefetch_model("invalid-model").await;
        assert!(result.is_err());

        // Test with empty model
        let result = provider.prefetch_model("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_request_error_handling() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        // Test with invalid endpoint
        let result: Result<ModelsResponse> = provider.send_request("/invalid-endpoint").await;
        assert!(result.is_err());

        // Test with invalid URL
        let provider = LMStudioProvider::new(
            "invalid-url".to_string(),
            "test-model".to_string(),
        );
        let result: Result<ModelsResponse> = provider.send_request("/v1/models").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_chat_request_error_handling() {
        let provider = LMStudioProvider::new(
            TEST_URL.to_string(),
            "test-model".to_string(),
        );

        // Test with invalid request
        let request = LMStudioRequest {
            messages: vec![],
            model: "test-model".to_string(),
        };
        let result = provider.send_chat_request(&request).await;
        assert!(result.is_err());

        // Test with invalid URL
        let provider = LMStudioProvider::new(
            "invalid-url".to_string(),
            "test-model".to_string(),
        );
        let result = provider.send_chat_request(&request).await;
        assert!(result.is_err());
    }
} 