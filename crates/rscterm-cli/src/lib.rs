use rscterm_core::{Provider, Error};
use rscterm_core::error::Result;
use colored::*;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};
use crate::commands::Command;
use crate::completion::AIbitatHelper;

pub mod commands;
pub mod completion;

pub struct CliInterface {
    current_team: Option<String>,
    show_progress: bool,
    history: Vec<String>,
    helper: AIbitatHelper,
}

impl CliInterface {
    pub fn new(_provider: Box<dyn Provider>) -> Self {
        Self {
            current_team: None,
            show_progress: true,
            history: Vec::new(),
            helper: AIbitatHelper::new(),
        }
    }

    pub fn get_current_team(&self) -> Option<&String> {
        self.current_team.as_ref()
    }

    pub fn set_current_team(&mut self, team: String) {
        self.current_team = Some(team);
    }

    pub fn get_show_progress(&self) -> bool {
        self.show_progress
    }

    pub fn set_show_progress(&mut self, show: bool) {
        self.show_progress = show;
    }

    pub fn print_message(&self, role: &str, message: &str) {
        let prefix = match role {
            "system" => "🤖".blue(),
            "user" => "👤".green(),
            "assistant" => "🤖".yellow(),
            _ => "💬".white(),
        };
        println!("{} {}", prefix, message);
    }

    pub fn print_success(&self, message: &str) {
        println!("{} {}", "✅".green(), message.green());
    }

    pub fn print_error(&self, message: &str) {
        println!("{} {}", "❌".red(), message.red());
    }

    pub fn clear_screen(&self) -> Result<()> {
        io::stdout()
            .execute(Clear(ClearType::All))
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }

    pub fn print_welcome(&self) {
        self.print_message("system", "Welcome to AIbitat! 🚀");
        self.print_message("system", "Type 'help' to see available commands");
        self.print_message("system", "Use TAB for command completion");
    }

    pub async fn simulate_agent_response(&self, agent: &str, message: &str) -> Result<()> {
        if self.show_progress {
            self.print_message(agent, message);
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        io::stdout()
            .execute(Hide)
            .map_err(|e| Error::Io(e))?;

        let mut input = String::new();
        let mut position = 0;
        let mut suggestions: Vec<String> = Vec::new();

        loop {
            if let Event::Key(key) = event::read().map_err(|e| Error::Io(e))? {
                match key {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => break,
                    KeyEvent {
                        code: KeyCode::Tab,
                        ..
                    } => {
                        if !suggestions.is_empty() {
                            input = suggestions[0].clone();
                            position = input.len();
                            suggestions.clear();
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if !input.trim().is_empty() {
                            self.history.push(input.clone());
                            let cmd = Command::from_input(&input);
                            if let Ok(should_continue) = cmd.execute(self).await {
                                if !should_continue {
                                    break;
                                }
                            }
                            input.clear();
                            position = 0;
                            suggestions.clear();
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        if position > 0 {
                            input.remove(position - 1);
                            position -= 1;
                            suggestions = self.helper.get_completions(&input);
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if position > 0 {
                            position -= 1;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if position < input.len() {
                            position += 1;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        input.insert(position, c);
                        position += 1;
                        suggestions = self.helper.get_completions(&input);
                    }
                    _ => {}
                }

                // Clear line and show prompt
                print!("\r\x1B[K🤖 AIbitat > {}", input);
                if !suggestions.is_empty() {
                    print!(" [{}]", suggestions.join(", ").blue());
                }
                io::stdout().flush().map_err(|e| Error::Io(e))?;
            }
        }

        io::stdout()
            .execute(Show)
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rscterm_provider::LMStudioProvider;

    #[tokio::test]
    async fn test_cli_creation() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        ));
        let cli = CliInterface::new(provider);
        assert!(cli.get_current_team().is_none());
        assert!(cli.get_show_progress());
    }

    #[tokio::test]
    async fn test_team_management() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        ));
        let mut cli = CliInterface::new(provider);
        cli.set_current_team("test-team".to_string());
        assert_eq!(cli.get_current_team(), Some(&"test-team".to_string()));
    }

    #[tokio::test]
    async fn test_progress_toggle() {
        let provider = Box::new(LMStudioProvider::new(
            "http://localhost:1234".to_string(),
            "test-model".to_string(),
        ));
        let mut cli = CliInterface::new(provider);
        let initial = cli.get_show_progress();
        cli.set_show_progress(!initial);
        assert_eq!(cli.get_show_progress(), !initial);
    }
} 