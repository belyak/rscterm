use aibitatr::{AIbitat, Agent, Channel, Message};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new conversation
    Start {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Send a message to a channel
    Send {
        /// Channel to send the message to
        #[arg(short, long)]
        channel: String,
        /// Message content
        #[arg(short, long)]
        message: String,
    },
    /// List all active channels
    Channels,
    /// List all available agents
    Agents,
}

fn load_config(path: Option<PathBuf>) -> Result<AIbitat> {
    let mut aibitat = AIbitat::new();
    
    if let Some(config_path) = path {
        let config = fs::read_to_string(config_path)?;
        let config: serde_json::Value = serde_json::from_str(&config)?;
        
        // Load agents
        if let Some(agents) = config.get("agents") {
            for agent in agents.as_array().unwrap_or(&Vec::new()) {
                let name = agent["name"].as_str().unwrap_or("unknown").to_string();
                let role = agent["role"].as_str().unwrap_or("").to_string();
                let interrupt_always = agent["interrupt_always"].as_bool().unwrap_or(false);
                let max_rounds = agent["max_rounds"].as_u64().map(|n| n as u32);
                
                let agent = Agent {
                    name: name.clone(),
                    role,
                    interrupt_always,
                    max_rounds,
                };
                
                aibitat = aibitat.with_agent(&name, agent);
            }
        }
        
        // Load channels
        if let Some(channels) = config.get("channels") {
            for channel in channels.as_array().unwrap_or(&Vec::new()) {
                let name = channel["name"].as_str().unwrap_or("unknown").to_string();
                let participants = channel["participants"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect();
                
                let channel = Channel {
                    name: name.clone(),
                    participants,
                };
                
                aibitat = aibitat.with_channel(&name, channel);
            }
        }
    } else {
        // Default configuration
        let assistant = Agent {
            name: "assistant".to_string(),
            role: "You are a helpful AI assistant.".to_string(),
            interrupt_always: true,
            max_rounds: Some(10),
        };
        
        let user = Agent {
            name: "user".to_string(),
            role: "You are a human user.".to_string(),
            interrupt_always: true,
            max_rounds: None,
        };
        
        let channel = Channel {
            name: "general".to_string(),
            participants: vec!["assistant".to_string(), "user".to_string()],
        };
        
        aibitat = aibitat
            .with_agent("assistant", assistant)
            .with_agent("user", user)
            .with_channel("general", channel);
    }
    
    Ok(aibitat)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { config } => {
            let _aibitat = load_config(config)?;
            println!("Started new conversation with configuration");
            // TODO: Implement interactive mode
        }
        Commands::Send { channel, message } => {
            let aibitat = load_config(None)?;
            let msg = Message {
                from: "user".to_string(),
                to: channel,
                content: message,
            };
            println!("Sending message: {}", msg.content);
            aibitat.start(msg).await?;
        }
        Commands::Channels => {
            let aibitat = load_config(None)?;
            println!("Active channels:");
            for (name, channel) in aibitat.channels.iter() {
                println!("- {} (participants: {})", name, channel.participants.join(", "));
            }
        }
        Commands::Agents => {
            let aibitat = load_config(None)?;
            println!("Available agents:");
            for (name, agent) in aibitat.agents.iter() {
                println!("- {} (role: {})", name, agent.role);
                println!("  Interrupt always: {}", agent.interrupt_always);
                if let Some(max_rounds) = agent.max_rounds {
                    println!("  Max rounds: {}", max_rounds);
                }
            }
        }
    }
    
    Ok(())
} 