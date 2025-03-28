pub mod cli;
pub mod lmstudio;
pub mod mock;

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

// Re-export common types
pub use crate::llm::LLMProvider;
pub use lmstudio::LMStudioProvider;
pub use mock::MockLLMProvider;

mod llm {
    use super::*;

    #[async_trait]
    pub trait LLMProvider: Send + Sync + std::fmt::Debug {
        async fn generate_response(&self, prompt: &str) -> Result<String>;
    }
}

#[derive(Debug, Error)]
pub enum AIbitatError {
    #[error("Agent error: {0}")]
    AgentError(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("LLM error: {0}")]
    LLMError(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AIbitatError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub role: String,
    pub interrupt_always: bool,
    pub max_rounds: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub name: String,
    pub participants: Vec<String>,
}

pub struct AIbitat {
    pub agents: HashMap<String, Agent>,
    pub channels: HashMap<String, Channel>,
    llm_provider: Box<dyn LLMProvider>,
}

impl AIbitat {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            channels: HashMap::new(),
            #[cfg(test)]
            llm_provider: Box::new(LMStudioProvider::with_emulation(
                "http://localhost:1234".to_string(),
                "test-model".to_string(),
            )),
            #[cfg(not(test))]
            llm_provider: Box::new(LMStudioProvider::default()),
        }
    }

    pub fn with_agent(mut self, name: &str, agent: Agent) -> Self {
        self.agents.insert(name.to_string(), agent);
        self
    }

    pub fn with_channel(mut self, name: &str, channel: Channel) -> Self {
        self.channels.insert(name.to_string(), channel);
        self
    }

    pub async fn start(&self, message: Message) -> Result<()> {
        // Verify sender exists
        if !self.agents.contains_key(&message.from) {
            return Err(AIbitatError::AgentError(format!("Sender '{}' not found", message.from)));
        }

        // Verify channel exists and sender is a participant
        if let Some(channel) = self.channels.get(&message.to) {
            if !channel.participants.contains(&message.from) {
                return Err(AIbitatError::ChannelError(format!(
                    "Sender '{}' is not a participant in channel '{}'",
                    message.from, message.to
                )));
            }
        } else {
            return Err(AIbitatError::ChannelError(format!(
                "Channel '{}' not found",
                message.to
            )));
        }

        // Generate response using LLM provider
        let _response = self.llm_provider.generate_response(&message.content).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_message_flow() {
        let aibitat = AIbitat::new()
            .with_agent("planner", Agent {
                name: "planner".to_string(),
                role: "planner".to_string(),
                interrupt_always: false,
                max_rounds: None,
            })
            .with_channel("test-channel", Channel {
                name: "test-channel".to_string(),
                participants: vec!["planner".to_string()],
            });

        let message = Message {
            from: "planner".to_string(),
            to: "test-channel".to_string(),
            content: "You are planner, an AI agent. Your task: Test task.".to_string(),
        };

        let result = aibitat.start(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_sender() {
        let aibitat = AIbitat::new()
            .with_channel("test-channel", Channel {
                name: "test-channel".to_string(),
                participants: vec!["planner".to_string()],
            });

        let message = Message {
            from: "invalid".to_string(),
            to: "test-channel".to_string(),
            content: "Test message".to_string(),
        };

        let result = aibitat.start(message).await;
        assert!(matches!(result, Err(AIbitatError::AgentError(_))));
    }

    #[tokio::test]
    async fn test_invalid_channel() {
        let aibitat = AIbitat::new()
            .with_agent("planner", Agent {
                name: "planner".to_string(),
                role: "planner".to_string(),
                interrupt_always: false,
                max_rounds: None,
            });

        let message = Message {
            from: "planner".to_string(),
            to: "invalid".to_string(),
            content: "Test message".to_string(),
        };

        let result = aibitat.start(message).await;
        assert!(matches!(result, Err(AIbitatError::ChannelError(_))));
    }

    #[tokio::test]
    async fn test_sender_not_in_channel() {
        let aibitat = AIbitat::new()
            .with_agent("planner", Agent {
                name: "planner".to_string(),
                role: "planner".to_string(),
                interrupt_always: false,
                max_rounds: None,
            })
            .with_channel("test-channel", Channel {
                name: "test-channel".to_string(),
                participants: vec!["other".to_string()],
            });

        let message = Message {
            from: "planner".to_string(),
            to: "test-channel".to_string(),
            content: "Test message".to_string(),
        };

        let result = aibitat.start(message).await;
        assert!(matches!(result, Err(AIbitatError::ChannelError(_))));
    }
}
