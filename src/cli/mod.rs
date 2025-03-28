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

use rscterm_completion::AIbitatHelper;
use crate::{LLMProvider, Result as AIbitatResult};

const AGENT_TOPICS: &[&str] = &[
    "planner",
    "researcher",
    "programmer",
    "designer",
    "reviewer",
];

pub struct CliInterface {
    prompt: String,
    history: Vec<String>,
    editor: Editor<AIbitatHelper, DefaultHistory>,
    current_team: Option<String>,
    multi_progress: MultiProgress,
    show_progress: bool,
    llm_provider: Box<dyn LLMProvider>,
    current_task: Option<String>,
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
            current_task: None,
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

    pub fn get_current_team(&self) -> Option<&String> {
        self.current_team.as_ref()
    }

    pub fn set_current_team(&mut self, team: String) {
        self.current_team = Some(team.clone());
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_team(team);
        }
        self.display_agent_communications();
    }

    pub fn add_task(&mut self, task: String) {
        self.current_task = Some(task.clone());
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_task(task);
        }
        self.display_agent_communications();
    }

    fn display_agent_communications(&self) {
        if let Some(task) = &self.current_task {
            println!("\n{}", "🤖 Agent Communications:".green().bold());
            println!("{}", "=".repeat(50).cyan());
            
            for agent in AGENT_TOPICS {
                let pb = self.multi_progress.add(ProgressBar::new(100));
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("=>-"));
                
                pb.set_message(format!("{}: Initializing...", agent));
                pb.set_position(30);

                let prompt = format!(
                    "You are {}, an AI agent. Your task: {}. Respond with your approach and initial steps.",
                    agent, task
                );

                // Simulate progress and show agent's initial response
                for i in 30..=100 {
                    pb.set_position(i);
                    if i == 60 {
                        pb.set_message(format!("{}: {}", agent, prompt));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }

                pb.finish_with_message(format!("{}: Ready to assist with task", agent));
            }
            
            println!("{}", "=".repeat(50).cyan());
            println!();
        }
    }

    pub async fn update_models(&mut self) -> AIbitatResult<()> {
        let models = self.llm_provider.list_models().await?;
        if let Some(helper) = self.editor.helper_mut() {
            helper.update_models(models);
        }
        Ok(())
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

    #[tokio::test]
    async fn test_cli_interface() {
        let mock_provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(mock_provider));

        // Test team management
        cli.set_current_team("test-team".to_string());
        assert_eq!(cli.current_team, Some("test-team".to_string()));

        // Test task management
        cli.add_task("test-task".to_string());
        assert_eq!(cli.current_task, Some("test-task".to_string()));

        // Test progress bar toggle
        cli.set_show_progress(false);
        assert!(!cli.get_show_progress());
        cli.set_show_progress(true);
        assert!(cli.get_show_progress());

        // Verify agent topics
        assert_eq!(AGENT_TOPICS.len(), 5);
        assert!(AGENT_TOPICS.contains(&"planner"));
        assert!(AGENT_TOPICS.contains(&"researcher"));
        assert!(AGENT_TOPICS.contains(&"programmer"));
        assert!(AGENT_TOPICS.contains(&"designer"));
        assert!(AGENT_TOPICS.contains(&"reviewer"));
    }
} 