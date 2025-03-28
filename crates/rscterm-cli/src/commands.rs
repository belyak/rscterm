use rscterm_core::error::Result;
use tracing::info;

#[derive(Debug)]
pub enum Command {
    Help,
    Exit,
    Clear,
    History,
    Send { channel: String, message: String },
    List,
    Agents,
    Channels,
    SetupTeam { name: String, agents: Vec<String> },
    StartTask { task: String },
    ToggleProgress,
    Chat { message: String },
    ListModels,
    SetModel { model: String },
    Unknown(String),
}

impl Command {
    pub fn from_input(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.get(0).map(|s| *s) {
            Some("help") => Command::Help,
            Some("exit") => Command::Exit,
            Some("clear") => Command::Clear,
            Some("history") => Command::History,
            Some("send") => {
                if parts.len() < 3 {
                    Command::Send {
                        channel: "".to_string(),
                        message: "".to_string(),
                    }
                } else {
                    let channel = parts[1].to_string();
                    let message = parts[2..].join(" ");
                    Command::Send { channel, message }
                }
            }
            Some("list") => Command::List,
            Some("agents") => Command::Agents,
            Some("channels") => Command::Channels,
            Some("setup-team") => {
                if parts.len() < 3 {
                    Command::SetupTeam {
                        name: "".to_string(),
                        agents: vec![],
                    }
                } else {
                    let name = parts[1].to_string();
                    let agents = parts[2..].iter().map(|s| s.to_string()).collect();
                    Command::SetupTeam { name, agents }
                }
            }
            Some("start-task") => {
                if parts.len() < 2 {
                    Command::StartTask {
                        task: "".to_string(),
                    }
                } else {
                    let task = parts[1..].join(" ");
                    Command::StartTask { task }
                }
            }
            Some("toggle-progress") => Command::ToggleProgress,
            Some("chat") => {
                if parts.len() < 2 {
                    Command::Chat {
                        message: "".to_string(),
                    }
                } else {
                    let message = parts[1..].join(" ");
                    Command::Chat { message }
                }
            }
            Some("list-models") => Command::ListModels,
            Some("set-model") => {
                if parts.len() < 2 {
                    Command::SetModel {
                        model: "".to_string(),
                    }
                } else {
                    let model = parts[1].to_string();
                    Command::SetModel { model }
                }
            }
            Some(cmd) => Command::Unknown(cmd.to_string()),
            None => Command::Unknown("".to_string()),
        }
    }

    pub async fn execute(&self, cli: &mut super::CliInterface) -> Result<bool> {
        match self {
            Command::Help => {
                cli.print_message("system", "Available Commands:");
                cli.print_message("system", "  help           - Show this help message");
                cli.print_message("system", "  clear          - Clear the screen");
                cli.print_message("system", "  history        - Show command history");
                cli.print_message("system", "  send           - Send message to a channel (usage: send <channel> <message>)");
                cli.print_message("system", "  list           - List available resources");
                cli.print_message("system", "  agents         - List all agents");
                cli.print_message("system", "  channels       - List all channels");
                cli.print_message("system", "  setup-team     - Create a new team (usage: setup-team <n> <agent1> <agent2> ...)");
                cli.print_message("system", "  start-task     - Start a new task (usage: start-task <task description>)");
                cli.print_message("system", "  chat           - Start a direct chat (usage: chat <message>)");
                cli.print_message("system", "  toggle-progress - Toggle progress display");
                cli.print_message("system", "  list-models    - Show available models in LM Studio");
                cli.print_message("system", "  set-model      - Change the current model (usage: set-model <model-name>)");
                cli.print_message("system", "  exit           - Exit the program");
                Ok(true)
            }
            Command::Exit => {
                cli.print_success("Goodbye! 👋");
                Ok(false)
            }
            Command::Clear => {
                cli.clear_screen()?;
                cli.print_welcome();
                Ok(true)
            }
            Command::History => {
                cli.print_message("system", "Command History:");
                for (i, cmd) in cli.history.iter().enumerate() {
                    cli.print_message("system", &format!("  {}. {}", i + 1, cmd));
                }
                Ok(true)
            }
            Command::Send { channel, message } => {
                if channel.is_empty() || message.is_empty() {
                    cli.print_error("Usage: send <channel> <message>");
                    cli.print_message("system", "Example: send #general Hello world");
                    Ok(true)
                } else {
                    cli.print_message("system", &format!("Sending to {}: {}", channel, message));
                    // TODO: Implement actual message sending
                    Ok(true)
                }
            }
            Command::List => {
                cli.print_message("system", "Available Resources:");
                cli.print_message("system", "  - agents");
                cli.print_message("system", "  - channels");
                cli.print_message("system", "  - teams");
                Ok(true)
            }
            Command::Agents => {
                cli.print_message("system", "Available Agents:");
                cli.print_message("system", "  - researcher (Expert in gathering and analyzing information)");
                cli.print_message("system", "  - programmer (Expert in coding and implementation)");
                cli.print_message("system", "  - designer (Expert in UI/UX and visual design)");
                cli.print_message("system", "  - planner (Expert in project planning and organization)");
                cli.print_message("system", "  - reviewer (Expert in code review and quality assurance)");
                Ok(true)
            }
            Command::Channels => {
                cli.print_message("system", "Available Channels:");
                cli.print_message("system", "  - #general (General discussion)");
                cli.print_message("system", "  - #tasks (Task-related discussions)");
                cli.print_message("system", "  - #code (Code-related discussions)");
                Ok(true)
            }
            Command::SetupTeam { name, agents } => {
                if name.is_empty() || agents.is_empty() {
                    cli.print_error("Usage: setup-team <n> <agent1> <agent2> ...");
                    cli.print_message("system", "Example: setup-team web-team researcher programmer designer");
                    Ok(true)
                } else {
                    info!("Setting up team '{}' with agents: {:?}", name, agents);
                    cli.print_success(&format!("Creating team '{}' with agents: {}", name, agents.join(", ")));
                    cli.set_current_team(name.clone());
                    
                    // Use LLM for team setup responses
                    cli.simulate_agent_response("planner", &format!("Setting up team '{}' with {} agents", name, agents.len())).await?;
                    for agent in agents {
                        cli.simulate_agent_response(&agent, &format!("Ready to work on team '{}'", name)).await?;
                    }
                    
                    Ok(true)
                }
            }
            Command::StartTask { task } => {
                if task.is_empty() {
                    cli.print_error("Usage: start-task <task description>");
                    cli.print_message("system", "Example: start-task Create a responsive web application");
                    Ok(true)
                } else {
                    if let Some(_team) = cli.get_current_team() {
                        info!("Starting task: {}", task);
                        cli.print_success(&format!("Starting task: {}", task));
                        
                        // Use LLM for task execution responses
                        cli.simulate_agent_response("planner", &format!("Breaking down task: {}", task)).await?;
                        cli.simulate_agent_response("researcher", "Gathering requirements and best practices").await?;
                        cli.simulate_agent_response("designer", "Creating initial design concepts").await?;
                        cli.simulate_agent_response("programmer", "Setting up project structure").await?;
                        cli.simulate_agent_response("reviewer", "Reviewing initial setup").await?;
                        
                        Ok(true)
                    } else {
                        cli.print_error("No active team. Please create a team first using 'setup-team'");
                        Ok(true)
                    }
                }
            }
            Command::ToggleProgress => {
                let current = cli.get_show_progress();
                cli.set_show_progress(!current);
                cli.print_success(&format!("Progress display {}", if !current { "enabled" } else { "disabled" }));
                Ok(true)
            }
            Command::Chat { message } => {
                if message.is_empty() {
                    cli.print_error("Usage: chat <message>");
                    cli.print_message("system", "Example: chat What is the best way to structure a web application?");
                    Ok(true)
                } else {
                    info!("Starting direct chat: {}", message);
                    cli.print_success(&format!("Starting chat: {}", message));
                    
                    // Use LLM for direct chat response
                    cli.simulate_agent_response("assistant", &message).await?;
                    
                    Ok(true)
                }
            }
            Command::ListModels => {
                cli.print_message("system", "Available Models in LM Studio:");
                cli.print_message("system", "  - gemma-3-12b-it (Gemma 3 12B Instruct)");
                cli.print_message("system", "  - gemma-2b-it (Gemma 2B Instruct)");
                cli.print_message("system", "  - mistral-7b (Mistral 7B)");
                cli.print_message("system", "  - llama-2-7b (Llama 2 7B)");
                cli.print_message("system", "  - codellama-7b (CodeLlama 7B)");
                Ok(true)
            }
            Command::SetModel { model } => {
                if model.is_empty() {
                    cli.print_error("Usage: set-model <model-name>");
                    cli.print_message("system", "Example: set-model gemma-3-12b-it");
                    Ok(true)
                } else {
                    let valid_models = vec![
                        "gemma-3-12b-it",
                        "gemma-2b-it",
                        "mistral-7b",
                        "llama-2-7b",
                        "codellama-7b"
                    ];
                    
                    if valid_models.contains(&model.as_str()) {
                        cli.print_success(&format!("Switching to model: {}", model));
                        // TODO: Implement actual model switching in LMStudioProvider
                        Ok(true)
                    } else {
                        cli.print_error(&format!("Invalid model: {}. Use 'list-models' to see available models.", model));
                        Ok(true)
                    }
                }
            }
            Command::Unknown(cmd) => {
                if !cmd.is_empty() {
                    cli.print_error(&format!("Unknown command: {}", cmd));
                    cli.print_message("system", "Type 'help' to see available commands");
                }
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rscterm_provider::LMStudioProvider;

    #[test]
    fn test_command_parsing() {
        // Test basic commands
        assert!(matches!(Command::from_input("help"), Command::Help));
        assert!(matches!(Command::from_input("exit"), Command::Exit));
        assert!(matches!(Command::from_input("clear"), Command::Clear));
        assert!(matches!(Command::from_input("history"), Command::History));
        assert!(matches!(Command::from_input("list"), Command::List));
        assert!(matches!(Command::from_input("agents"), Command::Agents));
        assert!(matches!(Command::from_input("channels"), Command::Channels));
        assert!(matches!(Command::from_input("toggle-progress"), Command::ToggleProgress));
        assert!(matches!(Command::from_input("list-models"), Command::ListModels));
        
        // Test send command
        if let Command::Send { channel, message } = Command::from_input("send #general Hello world") {
            assert_eq!(channel, "#general");
            assert_eq!(message, "Hello world");
        } else {
            panic!("Expected Send command");
        }

        // Test setup-team command
        if let Command::SetupTeam { name, agents } = Command::from_input("setup-team web-team researcher programmer") {
            assert_eq!(name, "web-team");
            assert_eq!(agents, vec!["researcher", "programmer"]);
        } else {
            panic!("Expected SetupTeam command");
        }
        
        // Test start-task command
        if let Command::StartTask { task } = Command::from_input("start-task Create a web app") {
            assert_eq!(task, "Create a web app");
        } else {
            panic!("Expected StartTask command");
        }

        // Test chat command
        if let Command::Chat { message } = Command::from_input("chat What is the best way to structure a web application?") {
            assert_eq!(message, "What is the best way to structure a web application?");
        } else {
            panic!("Expected Chat command");
        }

        // Test set-model command
        if let Command::SetModel { model } = Command::from_input("set-model gemma-3-12b-it") {
            assert_eq!(model, "gemma-3-12b-it");
        } else {
            panic!("Expected SetModel command");
        }
        
        // Test unknown commands
        assert!(matches!(Command::from_input("unknown"), Command::Unknown(_)));
        assert!(matches!(Command::from_input(""), Command::Unknown(_)));
    }

    #[test]
    fn test_empty_commands() {
        // Test empty send command
        if let Command::Send { channel, message } = Command::from_input("send") {
            assert!(channel.is_empty());
            assert!(message.is_empty());
        } else {
            panic!("Expected empty Send command");
        }

        // Test empty setup-team command
        if let Command::SetupTeam { name, agents } = Command::from_input("setup-team") {
            assert!(name.is_empty());
            assert!(agents.is_empty());
        } else {
            panic!("Expected empty SetupTeam command");
        }

        // Test empty start-task command
        if let Command::StartTask { task } = Command::from_input("start-task") {
            assert!(task.is_empty());
        } else {
            panic!("Expected empty StartTask command");
        }

        // Test empty chat command
        if let Command::Chat { message } = Command::from_input("chat") {
            assert!(message.is_empty());
        } else {
            panic!("Expected empty Chat command");
        }

        // Test empty set-model command
        if let Command::SetModel { model } = Command::from_input("set-model") {
            assert!(model.is_empty());
        } else {
            panic!("Expected empty SetModel command");
        }
    }

    #[tokio::test]
    async fn test_command_execution() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        ));
        let mut cli = super::CliInterface::new(provider);

        // Test help command
        let result = Command::Help.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test clear command
        let result = Command::Clear.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test history command
        let result = Command::History.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test setup-team command
        let cmd = Command::SetupTeam {
            name: "test-team".to_string(),
            agents: vec!["researcher".to_string(), "programmer".to_string()],
        };
        let result = cmd.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(cli.get_current_team(), Some(&"test-team".to_string()));

        // Test start-task command
        let cmd = Command::StartTask {
            task: "Test task".to_string(),
        };
        let result = cmd.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test chat command
        let cmd = Command::Chat {
            message: "Test message".to_string(),
        };
        let result = cmd.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test toggle-progress command
        let initial_progress = cli.get_show_progress();
        let result = Command::ToggleProgress.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_ne!(cli.get_show_progress(), initial_progress);

        // Test list-models command
        let result = Command::ListModels.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test set-model command with valid model
        let cmd = Command::SetModel {
            model: "gemma-3-12b-it".to_string(),
        };
        let result = cmd.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test set-model command with invalid model
        let cmd = Command::SetModel {
            model: "invalid-model".to_string(),
        };
        let result = cmd.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test exit command
        let result = Command::Exit.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_command_validation() {
        // Test invalid send command
        if let Command::Send { channel, message } = Command::from_input("send") {
            assert!(channel.is_empty());
            assert!(message.is_empty());
        } else {
            panic!("Expected empty Send command");
        }

        // Test invalid setup-team command
        if let Command::SetupTeam { name, agents } = Command::from_input("setup-team") {
            assert!(name.is_empty());
            assert!(agents.is_empty());
        } else {
            panic!("Expected empty SetupTeam command");
        }

        // Test invalid start-task command
        if let Command::StartTask { task } = Command::from_input("start-task") {
            assert!(task.is_empty());
        } else {
            panic!("Expected empty StartTask command");
        }

        // Test invalid chat command
        if let Command::Chat { message } = Command::from_input("chat") {
            assert!(message.is_empty());
        } else {
            panic!("Expected empty Chat command");
        }

        // Test invalid set-model command
        if let Command::SetModel { model } = Command::from_input("set-model") {
            assert!(model.is_empty());
        } else {
            panic!("Expected empty SetModel command");
        }
    }
} 