use async_trait::async_trait;
use crate::{LLMProvider, Result, AIbitatError};
use reqwest::Client;

#[derive(Debug)]
pub struct LMStudioProvider {
    client: Client,
    base_url: String,
    model: String,
    #[cfg(test)]
    emulate_responses: bool,
}

impl LMStudioProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
            #[cfg(test)]
            emulate_responses: false,
        }
    }

    #[cfg(test)]
    pub fn with_emulation(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
            emulate_responses: true,
        }
    }

    #[cfg(test)]
    fn emulate_response(&self, prompt: &str) -> Result<String> {
        // Extract agent role from prompt
        let agent = prompt
            .split_whitespace()
            .nth(2)
            .unwrap_or("unknown")
            .trim_end_matches(',');

        // Emulate responses based on agent role
        let response = match agent {
            "planner" => "I am the planner. I will help organize this task: breaking it down into steps, setting priorities, and creating a timeline.",
            "researcher" => "As the researcher, I will gather relevant information, analyze data, and provide evidence-based insights.",
            "programmer" => "I'm the programmer. I'll implement the technical solution, write clean code, and ensure it meets requirements.",
            "designer" => "As the designer, I'll focus on user experience, visual aesthetics, and creating intuitive interfaces.",
            "reviewer" => "I'm the reviewer. I'll evaluate the work, provide constructive feedback, and ensure quality standards are met.",
            _ => "I understand the task and will assist accordingly.",
        };

        Ok(response.to_string())
    }
}

impl Default for LMStudioProvider {
    fn default() -> Self {
        Self::new(
            "http://10.6.1.238:1234".to_string(),
            "gemma-3-12b-it".to_string(),
        )
    }
}

#[async_trait]
impl LLMProvider for LMStudioProvider {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        #[cfg(test)]
        if self.emulate_responses {
            return Ok(self.emulate_response(prompt)?);
        }

        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.7,
                "max_tokens": 1000
            }))
            .send()
            .await
            .map_err(|e| AIbitatError::LLMError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AIbitatError::LLMError(format!(
                "LM Studio API request failed: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| AIbitatError::LLMError(e.to_string()))?;
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| AIbitatError::LLMError("Invalid response format from LM Studio".to_string()))?;

        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emulated_responses() {
        let provider = LMStudioProvider::with_emulation(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );

        // Test planner response
        let response = provider
            .generate_response("You are planner, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("planner"));
        assert!(response.contains("organize"));

        // Test researcher response
        let response = provider
            .generate_response("You are researcher, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("researcher"));
        assert!(response.contains("gather"));

        // Test programmer response
        let response = provider
            .generate_response("You are programmer, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("programmer"));
        assert!(response.contains("implement"));

        // Test designer response
        let response = provider
            .generate_response("You are designer, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("designer"));
        assert!(response.contains("user experience"));

        // Test reviewer response
        let response = provider
            .generate_response("You are reviewer, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("reviewer"));
        assert!(response.contains("evaluate"));

        // Test unknown agent response
        let response = provider
            .generate_response("You are unknown, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert!(response.contains("understand"));
    }

    #[tokio::test]
    async fn test_real_provider_creation() {
        let provider = LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );
        assert_eq!(provider.base_url, "http://localhost:1234");
        assert_eq!(provider.model, "test-model");
        #[cfg(test)]
        assert!(!provider.emulate_responses);
    }
} 