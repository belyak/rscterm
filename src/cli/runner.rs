use anyhow::Result;
use tracing::{info, debug};

use crate::{
    cli::CliInterface,
    LMStudioProvider,
};

pub struct CliRunner {
    cli: CliInterface,
}

impl CliRunner {
    pub fn new() -> Self {
        #[cfg(test)]
        let provider = LMStudioProvider::with_emulation(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        );
        #[cfg(not(test))]
        let provider = LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "gemma-3-12b-it".to_string()
        );
        let cli = CliInterface::new(Box::new(provider));
        Self { cli }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.cli.clear_screen()?;
        self.cli.print_welcome();

        loop {
            let input = self.cli.get_input()?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            debug!("Processing command: {}", input);

            if input == "exit" {
                info!("Exiting CLI");
                break;
            }

            if let Err(e) = self.handle_command(input).await {
                self.cli.print_error(&format!("{}", e));
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, input: &str) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  help           - Show this help message");
                println!("  clear          - Clear the screen");
                println!("  setup-team     - Set up a new team");
                println!("  start-task     - Start a new task");
                println!("  exit           - Exit the CLI");
            }
            "clear" => {
                self.cli.clear_screen()?;
            }
            "setup-team" => {
                if parts.len() < 2 {
                    self.cli.print_error("Please provide a team name");
                    return Ok(());
                }
                let team_name = parts[1].to_string();
                self.cli.set_current_team(team_name.clone());
                self.cli.print_success(&format!("Team '{}' has been set up", team_name));
            }
            "start-task" => {
                if let Some(_team) = self.cli.get_current_team() {
                    if parts.len() < 2 {
                        self.cli.print_error("Please provide a task description");
                        return Ok(());
                    }
                    let task = parts[1..].join(" ");
                    self.cli.simulate_agent_response("Agent1", &task).await?;
                } else {
                    self.cli.print_error("Please set up a team first using 'setup-team <name>'");
                }
            }
            _ => {
                self.cli.print_error(&format!("Unknown command: {}", parts[0]));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_runner_creation() {
        let runner = CliRunner::new();
        assert!(runner.cli.get_current_team().is_none());
    }
} 