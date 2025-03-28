pub mod commands;

use commands::Command;
use rscterm_core::{Error, Provider};
use rscterm_core::error::Result;
use std::io::{self, Write, stdout};
use colored::*;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use rscterm_completion::AIbitatHelper;
use rustyline::completion::{Completer, Pair};
use rustyline::Context;
use rustyline::history::DefaultHistory;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use indicatif::{ProgressBar, ProgressStyle};

pub struct CliInterface {
    current_team: Option<String>,
    show_progress: bool,
    history: Vec<String>,
    helper: AIbitatHelper,
    provider: Box<dyn Provider>,
}

impl CliInterface {
    pub fn new(provider: Box<dyn Provider>) -> Self {
        Self {
            current_team: None,
            show_progress: true,
            history: Vec::new(),
            helper: AIbitatHelper::new(),
            provider,
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

    pub fn get_provider(&mut self) -> &mut Box<dyn Provider> {
        &mut self.provider
    }

    pub fn print_message(&self, from: &str, content: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("{} [{}] {}: {}", 
            timestamp.cyan(),
            from.green().bold(),
            "Message".yellow(),
            content
        );
    }

    pub fn print_error(&self, message: &str) {
        self.print_message("error", message);
    }

    pub fn print_success(&self, message: &str) {
        self.print_message("success", message);
    }

    pub fn clear_screen(&self) -> Result<()> {
        io::stdout().execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn print_welcome(&self) {
        println!("{}", "Welcome to AIbitat CLI!".green().bold());
        println!("{}", "Type 'exit' to quit, 'help' for commands".yellow());
        println!("{}", "Use TAB for command completion".cyan().italic());
        println!();
    }

    pub async fn simulate_agent_response(&self, agent: &str, message: &str) -> Result<()> {
        if !self.show_progress {
            self.print_message(agent, message);
            return Ok(());
        }

        let mut stdout = stdout();
        let progress = ProgressBar::new(100);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );

        let mut rng = rand::thread_rng();
        let mut current = 0;
        let total_steps = 100;
        let step_delay = Duration::from_millis(20);

        // Customize progress bar based on agent type
        let (color, prefix) = match agent {
            "planner" => ("yellow", "[PLAN]"),
            "researcher" => ("blue", "[RES]"),
            "programmer" => ("green", "[DEV]"),
            "designer" => ("magenta", "[UI]"),
            "reviewer" => ("cyan", "[REV]"),
            "assistant" => ("white", "[AI]"),
            _ => ("white", "[???]"),
        };

        progress.set_message(format!("{} {}: {}", prefix, agent, message));
        progress.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{spinner:.{}}} [{{bar:40.{}/blue}}] {{percent}}% {{msg}}", 
                    color,
                    color))
                .unwrap()
                .progress_chars("=>-"),
        );

        while current < total_steps {
            current += rng.gen_range(1..=5);
            current = current.min(total_steps);
            progress.set_position(current);
            stdout.flush().map_err(|e| Error::Io(e))?;
            sleep(step_delay).await;
        }

        progress.finish_with_message(format!("{} {}: {}", prefix, agent, message));
        stdout.flush().map_err(|e| Error::Io(e))?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        io::stdout()
            .execute(Hide)
            .map_err(|e| Error::Io(e))?;

        let mut input = String::new();
        let mut position = 0;
        let mut suggestions: Vec<Pair> = Vec::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

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
                            input = suggestions[0].replacement.clone();
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
                            if let Ok((_, comps)) = self.helper.complete(&input, position, &ctx) {
                                suggestions = comps;
                            }
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
                        if let Ok((_, comps)) = self.helper.complete(&input, position, &ctx) {
                            suggestions = comps;
                        }
                    }
                    _ => {}
                }

                // Clear line and show prompt
                print!("\r\x1B[K[RSC] > {}", input);
                if !suggestions.is_empty() {
                    let suggestions_str = suggestions.iter()
                        .map(|s| s.display.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    print!(" [{}]", suggestions_str.blue());
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