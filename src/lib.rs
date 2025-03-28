use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

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

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate_response(&self, prompt: &str) -> Result<String>;
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

#[derive(Default)]
pub struct LMStudioProvider {
    // TODO: Add configuration options for LM-Studio
}

#[async_trait]
impl LLMProvider for LMStudioProvider {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        // TODO: Implement actual LM-Studio integration
        // For now, return a mock response based on the prompt
        Ok(format!("Mock response to: {}", prompt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_message_flow() {
        let mut aibitat = AIbitat::new();
        
        // Create a test agent
        let agent = Agent {
            name: "test".to_string(),
            role: "test role".to_string(),
            interrupt_always: false,
            max_rounds: None,
        };
        
        // Create a test channel
        let channel = Channel {
            name: "test".to_string(),
            participants: vec!["test".to_string()],
        };
        
        // Set up the test environment
        aibitat = aibitat
            .with_agent("test", agent)
            .with_channel("test", channel);
        
        let message = Message {
            from: "test".to_string(),
            to: "test".to_string(),
            content: "test".to_string(),
        };
        
        assert!(aibitat.start(message).await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_sender() {
        let aibitat = AIbitat::new();
        
        let message = Message {
            from: "nonexistent".to_string(),
            to: "test".to_string(),
            content: "test".to_string(),
        };
        
        let result = aibitat.start(message).await;
        assert!(matches!(result, Err(AIbitatError::AgentError(_))));
    }
}
