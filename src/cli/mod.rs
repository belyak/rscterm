use std::io;
use colored::*;
use crossterm::{
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tracing::debug;

pub mod commands;
pub mod runner;
pub mod completion;

use completion::AIbitatHelper;
use crate::{LLMProvider, Result as AIbitatResult};

pub struct CliInterface {
    prompt: String,
    history: Vec<String>,
    editor: Editor<AIbitatHelper, DefaultHistory>,
    current_team: Option<String>,
    multi_progress: MultiProgress,
    show_progress: bool,
    llm_provider: Box<dyn LLMProvider>,
}

impl CliInterface {
    pub fn new(llm_provider: Box<dyn LLMProvider>) -> Self {
        let helper = AIbitatHelper::new();
        let mut editor = Editor::new().unwrap();
        editor.set_helper(Some(helper));
        let _ = editor.load_history("history.txt"); // Load history if exists

        Self {
            prompt: "🤖 AIbitat > ".to_string(),
            history: Vec::new(),
            editor,
            current_team: None,
            multi_progress: MultiProgress::new(),
            show_progress: true,
            llm_provider,
        }
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

    pub fn print_message(&self, from: &str, content: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("{} [{}] {}: {}", 
            timestamp.cyan(),
            from.green().bold(),
            "Message".yellow(),
            content
        );
    }

    pub fn get_input(&mut self) -> Result<String> {
        match self.editor.readline(&self.prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    let _ = self.editor.add_history_entry(line.as_str());
                    self.history.push(line.clone());
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                Ok("exit".to_string())
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                Ok("exit".to_string())
            }
            Err(err) => {
                println!("Error: {:?}", err);
                Ok("".to_string())
            }
        }
    }

    pub fn print_error(&self, error: &str) {
        println!("{} {}", "Error:".red().bold(), error.red());
    }

    pub fn print_success(&self, message: &str) {
        println!("{} {}", "Success:".green().bold(), message.green());
    }

    pub fn save_history(&mut self) -> Result<()> {
        self.editor.save_history("history.txt")?;
        Ok(())
    }

    pub fn set_current_team(&mut self, name: String) {
        self.current_team = Some(name.clone());
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_team(name);
        }
    }

    pub fn get_current_team(&self) -> Option<&String> {
        self.current_team.as_ref()
    }

    pub async fn simulate_agent_response(&self, agent: &str, message: &str) -> AIbitatResult<()> {
        debug!("Agent {} processing message: {}", agent, message);
        
        let prompt = format!(
            "You are {}, an AI agent. Your task: {}. Respond with your approach and initial steps.",
            agent, message
        );
        
        if self.show_progress {
            let pb = self.multi_progress.add(ProgressBar::new(100));
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"));
            
            pb.set_message(format!("{}: Thinking...", agent));
            pb.set_position(30);

            debug!("Sending request to LLM provider");
            let response = self.llm_provider.generate_response(&prompt).await?;
            debug!("Received response from LLM: {}", response);
            
            pb.set_position(60);
            pb.set_message(format!("{}: {}", agent, response));
            
            for i in 60..=100 {
                pb.set_position(i);
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            
            pb.finish_with_message(format!("{}: {}", agent, response));
        } else {
            debug!("Sending request to LLM provider without progress bar");
            let response = self.llm_provider.generate_response(&prompt).await?;
            debug!("Received response from LLM: {}", response);
            self.print_message(agent, &response);
        }

        Ok(())
    }

    pub fn set_show_progress(&mut self, show: bool) {
        self.show_progress = show;
    }

    pub fn get_show_progress(&self) -> bool {
        self.show_progress
    }
}

impl Drop for CliInterface {
    fn drop(&mut self) {
        self.save_history().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockLLMProvider;

    #[test]
    fn test_cli_interface_creation() {
        let provider = MockLLMProvider::new();
        let cli = CliInterface::new(Box::new(provider));
        assert_eq!(cli.prompt, "🤖 AIbitat > ");
        assert!(cli.history.is_empty());
        assert!(cli.current_team.is_none());
        assert!(cli.show_progress);
    }

    #[test]
    fn test_set_current_team() {
        let provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(provider));
        
        cli.set_current_team("test-team".to_string());
        assert_eq!(cli.get_current_team(), Some(&"test-team".to_string()));
    }

    #[test]
    fn test_toggle_progress() {
        let provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(provider));
        
        let initial_progress = cli.get_show_progress();
        cli.set_show_progress(!initial_progress);
        assert_ne!(cli.get_show_progress(), initial_progress);
    }

    #[test]
    fn test_add_to_history() {
        let provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(provider));
        
        cli.history.push("test command".to_string());
        assert_eq!(cli.history.len(), 1);
        assert_eq!(cli.history[0], "test command");
    }

    #[test]
    fn test_print_message() {
        let provider = MockLLMProvider::new();
        let cli = CliInterface::new(Box::new(provider));
        
        // Note: This test only verifies that the function doesn't panic
        cli.print_message("test", "test message");
        cli.print_error("test error");
        cli.print_success("test success");
    }

    #[test]
    fn test_clear_screen() {
        let provider = MockLLMProvider::new();
        let cli = CliInterface::new(Box::new(provider));
        
        // Note: This test only verifies that the function doesn't panic
        assert!(cli.clear_screen().is_ok());
    }

    #[test]
    fn test_print_welcome() {
        let provider = MockLLMProvider::new();
        let cli = CliInterface::new(Box::new(provider));
        
        // Note: This test only verifies that the function doesn't panic
        cli.print_welcome();
    }

    #[tokio::test]
    async fn test_simulate_agent_response() {
        let provider = MockLLMProvider::new();
        let cli = CliInterface::new(Box::new(provider));
        
        // Test with known agent
        let result = cli.simulate_agent_response("planner", "Test task").await;
        assert!(result.is_ok());

        // Test with unknown agent
        let result = cli.simulate_agent_response("unknown", "Test task").await;
        assert!(result.is_ok());
    }
} 