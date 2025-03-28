# AIbitatR

A minimalistic, compact, and high-performance Rust implementation of the AIbitat multi-agent conversation framework. This project focuses on core functionality while optimizing for speed and resource efficiency, with LM-Studio as the primary LLM provider integration.

## Features

- Multi-agent conversation framework
- Channel-based communication
- LM-Studio integration for LLM responses
- Configurable agent roles and behaviors
- Support for human interruption
- Event-driven architecture

## Installation

### Prerequisites

- Rust 2021 edition or later
- LM-Studio running locally (for LLM integration)

### Building from Source

```bash
git clone https://github.com/yourusername/AbitatR.git
cd AbitatR
cargo build --release
```

## Usage

### Basic Example

```rust
use aibitatr::{AIbitat, Agent, Channel, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut aibitat = AIbitat::new();
    
    // Create agents
    let assistant = Agent {
        name: "assistant".to_string(),
        role: "You are a helpful AI assistant.".to_string(),
        interrupt_always: true,
        max_rounds: Some(10),
    };
    
    // Set up environment
    aibitat = aibitat
        .with_agent("assistant", assistant)
        .with_channel("general", Channel {
            name: "general".to_string(),
            participants: vec!["assistant".to_string()],
        });
    
    // Send a message
    aibitat.start(Message {
        from: "user".to_string(),
        to: "general".to_string(),
        content: "Hello!".to_string(),
    }).await?;
    
    Ok(())
}
```

### CLI Usage

```bash
# Start a new conversation
aibitatr start

# Start with specific configuration
aibitatr start --config config.json

# Send a message to an existing conversation
aibitatr send --channel general --message "Hello!"

# List active channels
aibitatr channels

# List available agents
aibitatr agents
```

## Configuration

The framework can be configured using a JSON file:

```json
{
  "agents": [
    {
      "name": "assistant",
      "role": "You are a helpful AI assistant.",
      "interrupt_always": true,
      "max_rounds": 10
    }
  ],
  "channels": [
    {
      "name": "general",
      "participants": ["assistant"]
    }
  ],
  "llm": {
    "provider": "lmstudio",
    "url": "http://localhost:1234",
    "model": "local-model"
  }
}
```

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 