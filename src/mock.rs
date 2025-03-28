use async_trait::async_trait;
use crate::{LLMProvider, Result};

#[derive(Debug)]
pub struct MockLLMProvider {
    responses: std::collections::HashMap<String, String>,
}

impl MockLLMProvider {
    pub fn new() -> Self {
        let mut responses = std::collections::HashMap::new();
        responses.insert("planner".to_string(), "I'll help plan this task.".to_string());
        responses.insert("researcher".to_string(), "I'll research this topic.".to_string());
        responses.insert("programmer".to_string(), "I'll help with the programming.".to_string());
        responses.insert("designer".to_string(), "I'll help with the design.".to_string());
        responses.insert("reviewer".to_string(), "I'll review the work.".to_string());
        responses.insert("assistant".to_string(), "I'll assist with this task.".to_string());
        Self { responses }
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        // Extract agent name from prompt
        let agent = prompt
            .split_whitespace()
            .nth(2)
            .unwrap_or("unknown")
            .trim_end_matches(',');
        
        Ok(self.responses.get(agent).cloned().unwrap_or_else(|| {
            format!("I am {}, and I'll help with this task.", agent)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockLLMProvider::new();
        
        // Test with known agent
        let response = provider
            .generate_response("You are planner, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert_eq!(response, "I'll help plan this task.");

        // Test with unknown agent
        let response = provider
            .generate_response("You are unknown, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert_eq!(response, "I am unknown, and I'll help with this task.");

        // Test with custom agent
        let response = provider
            .generate_response("You are custom, an AI agent. Your task: Test task.")
            .await
            .unwrap();
        assert_eq!(response, "I am custom, and I'll help with this task.");
    }
} 