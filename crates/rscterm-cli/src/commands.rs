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
    Run,
    Unknown(String),
}

impl Command {
    pub fn from_input(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.get(0).map(|s| *s).unwrap_or("");
        
        match cmd {
            "help" => Command::Help,
            "exit" => Command::Exit,
            "clear" => Command::Clear,
            "history" => Command::History,
            "send" => {
                if parts.len() < 3 {
                    Command::Send { channel: String::new(), message: String::new() }
                } else {
                    Command::Send {
                        channel: parts[1].to_string(),
                        message: parts[2..].join(" "),
                    }
                }
            }
            "list" => Command::List,
            "agents" => Command::Agents,
            "channels" => Command::Channels,
            "setup-team" => {
                if parts.len() < 3 {
                    Command::SetupTeam { name: String::new(), agents: vec![] }
                } else {
                    Command::SetupTeam {
                        name: parts[1].to_string(),
                        agents: parts[2..].iter().map(|s| s.to_string()).collect(),
                    }
                }
            }
            "start-task" => {
                if parts.len() < 2 {
                    Command::StartTask { task: String::new() }
                } else {
                    Command::StartTask { task: parts[1..].join(" ") }
                }
            }
            "toggle-progress" => Command::ToggleProgress,
            "chat" => {
                if parts.len() < 2 {
                    Command::Chat { message: String::new() }
                } else {
                    Command::Chat { message: parts[1..].join(" ") }
                }
            }
            "list-models" => Command::ListModels,
            "set-model" => {
                if parts.len() < 2 {
                    Command::SetModel { model: String::new() }
                } else {
                    Command::SetModel { model: parts[1].to_string() }
                }
            }
            "run" => Command::Run,
            cmd => Command::Unknown(cmd.to_string()),
        }
    }

    pub async fn execute(&self, cli: &mut super::CliInterface) -> Result<bool> {
        match self {
            Command::Help => {
                cli.print_message("system", "Available Commands:");
                for (cmd, desc) in HELP_MESSAGES {
                    cli.print_message("system", &format!("  {:<15} - {}", cmd, desc));
                }
                Ok(true)
            }
            Command::Exit => {
                cli.print_success("Goodbye!");
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
                    Ok(true)
                }
            }
            Command::List => {
                cli.print_message("system", "Available Resources:");
                for resource in &["agents", "channels", "teams"] {
                    cli.print_message("system", &format!("  - {}", resource));
                }
                Ok(true)
            }
            Command::Agents => {
                cli.print_message("system", "Available Agents:");
                for (agent, desc) in AGENT_DESCRIPTIONS {
                    cli.print_message("system", &format!("  - {} ({})", agent, desc));
                }
                Ok(true)
            }
            Command::Channels => {
                cli.print_message("system", "Available Channels:");
                for (channel, desc) in CHANNEL_DESCRIPTIONS {
                    cli.print_message("system", &format!("  - {} ({})", channel, desc));
                }
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
                } else if let Some(_team) = cli.get_current_team() {
                    info!("Starting task: {}", task);
                    cli.print_success(&format!("Starting task: {}", task));
                    
                    for (agent, msg) in TASK_START_MESSAGES {
                        cli.simulate_agent_response(agent, msg).await?;
                    }
                    
                    Ok(true)
                } else {
                    cli.print_error("No active team. Please create a team first using 'setup-team'");
                    Ok(true)
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
                    cli.simulate_agent_response("assistant", &message).await?;
                    Ok(true)
                }
            }
            Command::ListModels => {
                match cli.get_provider().get_loaded_models().await {
                    Ok(models) => {
                        cli.print_message("system", "Available Models in LM Studio:");
                        for model in models {
                            cli.print_message("system", &format!("  - {}", model));
                        }
                        Ok(true)
                    }
                    Err(e) => {
                        cli.print_error(&format!("Failed to fetch models: {}", e));
                        Ok(true)
                    }
                }
            }
            Command::SetModel { model } => {
                if model.is_empty() {
                    cli.print_error("Usage: set-model <model-name>");
                    cli.print_message("system", "Example: set-model gemma-3-12b-it");
                    Ok(true)
                } else {
                    match cli.get_provider().set_model(&model).await {
                        Ok(_) => {
                            cli.print_success(&format!("Switched to model: {}", model));
                            Ok(true)
                        }
                        Err(e) => {
                            cli.print_error(&format!("Failed to switch model: {}", e));
                            Ok(true)
                        }
                    }
                }
            }
            Command::Run => {
                let current_model = cli.get_provider().get_current_model();
                cli.print_success(&format!("Running model {} in terminal mode...", current_model));
                
                match cli.get_provider().run_terminal().await {
                    Ok(_) => {
                        cli.print_success("Terminal session ended");
                        Ok(true)
                    }
                    Err(e) => {
                        cli.print_error(&format!("Failed to run terminal mode: {}", e));
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

const HELP_MESSAGES: &[(&str, &str)] = &[
    ("help", "Show this help message"),
    ("clear", "Clear the screen"),
    ("history", "Show command history"),
    ("send", "Send message to a channel (usage: send <channel> <message>)"),
    ("list", "List available resources"),
    ("agents", "List all agents"),
    ("channels", "List all channels"),
    ("setup-team", "Create a new team (usage: setup-team <n> <agent1> <agent2> ...)"),
    ("start-task", "Start a new task (usage: start-task <task description>)"),
    ("chat", "Start a direct chat (usage: chat <message>)"),
    ("toggle-progress", "Toggle progress display"),
    ("list-models", "Show available models in LM Studio"),
    ("set-model", "Change the current model (usage: set-model <model-name>)"),
    ("run", "Run the current model in terminal mode"),
    ("exit", "Exit the program"),
];

const AGENT_DESCRIPTIONS: &[(&str, &str)] = &[
    ("researcher", "Expert in gathering and analyzing information"),
    ("programmer", "Expert in coding and implementation"),
    ("designer", "Expert in UI/UX and visual design"),
    ("planner", "Expert in project planning and organization"),
    ("reviewer", "Expert in code review and quality assurance"),
];

const CHANNEL_DESCRIPTIONS: &[(&str, &str)] = &[
    ("#general", "General discussion"),
    ("#tasks", "Task-related discussions"),
    ("#code", "Code-related discussions"),
];

const TASK_START_MESSAGES: &[(&str, &str)] = &[
    ("planner", "Breaking down task"),
    ("researcher", "Gathering requirements and best practices"),
    ("designer", "Creating initial design concepts"),
    ("programmer", "Setting up project structure"),
    ("reviewer", "Reviewing initial setup"),
];

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

    #[tokio::test]
    async fn test_model_commands() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "gemma-3-12b-it".to_string(),
        ));
        let mut cli = super::CliInterface::new(provider);

        // Test list-models command
        let result = Command::ListModels.execute(&mut cli).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test set-model command with empty model
        let cmd = Command::SetModel {
            model: "".to_string(),
        };
        let result = cmd.execute(&mut cli).await;
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

        // Test model switching sequence
        let valid_models = vec![
            "gemma-3-12b-it",
            "gemma-2b-it",
            "mistral-7b",
            "llama-2-7b",
            "codellama-7b",
        ];

        for model in valid_models {
            let cmd = Command::SetModel {
                model: model.to_string(),
            };
            let result = cmd.execute(&mut cli).await;
            assert!(result.is_ok());
            assert!(result.unwrap());
        }
    }

    #[tokio::test]
    async fn test_model_command_parsing() {
        // Test list-models command parsing
        assert!(matches!(Command::from_input("list-models"), Command::ListModels));

        // Test set-model command parsing with valid model
        if let Command::SetModel { model } = Command::from_input("set-model gemma-3-12b-it") {
            assert_eq!(model, "gemma-3-12b-it");
        } else {
            panic!("Expected SetModel command");
        }

        // Test set-model command parsing with empty model
        if let Command::SetModel { model } = Command::from_input("set-model") {
            assert!(model.is_empty());
        } else {
            panic!("Expected SetModel command with empty model");
        }

        // Test set-model command parsing with multiple words
        if let Command::SetModel { model } = Command::from_input("set-model invalid model name") {
            assert_eq!(model, "invalid");
        } else {
            panic!("Expected SetModel command");
        }
    }

    #[tokio::test]
    async fn test_model_interaction_with_other_commands() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "gemma-3-12b-it".to_string(),
        ));
        let mut cli = super::CliInterface::new(provider);

        // Set up a team and start a task
        let setup_cmd = Command::SetupTeam {
            name: "test-team".to_string(),
            agents: vec!["researcher".to_string(), "programmer".to_string()],
        };
        assert!(setup_cmd.execute(&mut cli).await.unwrap());

        // Switch model
        let model_cmd = Command::SetModel {
            model: "mistral-7b".to_string(),
        };
        assert!(model_cmd.execute(&mut cli).await.unwrap());

        // Start a task with the new model
        let task_cmd = Command::StartTask {
            task: "Test task".to_string(),
        };
        assert!(task_cmd.execute(&mut cli).await.unwrap());

        // Send a chat message with the new model
        let chat_cmd = Command::Chat {
            message: "Test message".to_string(),
        };
        assert!(chat_cmd.execute(&mut cli).await.unwrap());
    }
} 