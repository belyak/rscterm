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
use std::process::Command;
use std::path::PathBuf;

pub mod commands;
pub mod runner;

use rscterm_completion::CLIHelper;
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
    editor: Editor<CLIHelper, DefaultHistory>,
    current_team: Option<String>,
    multi_progress: MultiProgress,
    show_progress: bool,
    llm_provider: Box<dyn LLMProvider>,
    current_task: Option<String>,
    current_branch: Option<String>,
    #[cfg(test)]
    test_dir: Option<tempfile::TempDir>,
}

impl CliInterface {
    pub fn new(llm_provider: Box<dyn LLMProvider>) -> Self {
        let helper = CLIHelper::new();
        let mut editor = Editor::new().unwrap();
        editor.set_helper(Some(helper));
        let _ = editor.load_history("history.txt"); // Load history if exists

        Self {
            prompt: "🤖 > ".to_string(),
            history: Vec::new(),
            editor,
            current_team: None,
            multi_progress: MultiProgress::new(),
            show_progress: true,
            llm_provider,
            current_task: None,
            current_branch: None,
            #[cfg(test)]
            test_dir: None,
        }
    }

    #[cfg(test)]
    fn init_test_repo(&mut self) {
        // Clean up any existing test directory
        if let Some(dir) = self.test_dir.take() {
            let _ = fs::remove_dir_all(dir.path());
        }

        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        std::env::set_current_dir(&dir_path).unwrap();

        // Initialize git repository
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["init"])
            .output()
            .unwrap();

        // Configure git user
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["config", "user.name", "Test User"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["config", "user.email", "test@example.com"])
            .output()
            .unwrap();

        // Create and commit an initial file
        std::fs::write(dir_path.join("README.md"), "# Test Repository").unwrap();
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["add", "README.md"])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["commit", "-m", "Initial commit"])
            .output()
            .unwrap();

        // Create initial master branch
        Command::new("git")
            .current_dir(&dir_path)
            .args(&["checkout", "-b", "master"])
            .output()
            .unwrap();

        self.test_dir = Some(dir);
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
            helper.add_team(team.clone());
        }
        
        // Create master branch for the team
        self.create_team_branch(&team);
        
        self.display_agent_communications();
    }

    pub fn add_task(&mut self, task: String) {
        self.current_task = Some(task.clone());
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_task(task.clone());
        }
        
        // Create feature branch for the task
        if let Some(team) = self.current_team.clone() {
            self.create_task_branch(&team, &task);
        }
        
        self.display_agent_communications();
    }

    fn create_team_branch(&mut self, team: &str) {
        let branch_name = format!("master-{}", team.to_lowercase().replace(" ", "-"));
        
        // Try to switch to the branch if it exists, or create it
        let result = self.run_git_command(&["checkout", &branch_name]);
        if result.is_err() {
            // Branch doesn't exist, create it from master
            if let Err(_) = self.run_git_command(&["checkout", "master"]) {
                println!("{} Failed to switch to master branch", "❌".red());
                return;
            }
            if let Err(e) = self.run_git_command(&["checkout", "-b", &branch_name]) {
                println!("{} Failed to create team branch: {}", "❌".red(), e);
                return;
            }
        }
        
        println!("{} Created/switched to team branch: {}", "✅".green(), branch_name.cyan());
        self.current_branch = Some(branch_name);
    }

    fn create_task_branch(&mut self, team: &str, task: &str) {
        let branch_name = format!("feature/{}-{}", 
            team.to_lowercase().replace(" ", "-"),
            task.to_lowercase().replace(" ", "-")
        );
        
        // Try to switch to the branch if it exists, or create it
        let result = self.run_git_command(&["checkout", &branch_name]);
        if result.is_err() {
            // Branch doesn't exist, create it
            if let Err(e) = self.run_git_command(&["checkout", "-b", &branch_name]) {
                println!("{} Failed to create task branch: {}", "❌".red(), e);
                return;
            }
        }

        println!("{} Created/switched to task branch: {}", "✅".green(), branch_name.cyan());
        self.current_branch = Some(branch_name);
    }

    pub fn complete_task(&mut self) {
        let task = self.current_task.take();
        if let Some(task) = task {
            if let Some(team) = &self.current_team {
                let master_branch = format!("master-{}", team.to_lowercase().replace(" ", "-"));
                
                // First, make sure we're on the feature branch
                if let Some(feature_branch) = &self.current_branch {
                    // Try to switch to master branch
                    if let Err(e) = self.run_git_command(&["checkout", &master_branch]) {
                        println!("{} Failed to switch to master branch: {}", "❌".red(), e);
                        return;
                    }

                    // Merge the feature branch
                    if let Err(e) = self.run_git_command(&["merge", feature_branch]) {
                        println!("{} Failed to merge feature branch: {}", "❌".red(), e);
                        return;
                    }
                    println!("{} Merged feature branch: {}", "✅".green(), feature_branch.cyan());
                }

                // Update current branch state
                self.current_branch = Some(master_branch);
                if let Some(helper) = self.editor.helper_mut() {
                    helper.remove_task(&task);
                }
            }
        }
    }

    fn run_git_command(&self, args: &[&str]) -> Result<()> {
        #[cfg(test)]
        let current_dir = if let Some(dir) = self.test_dir.as_ref() {
            dir.path().to_path_buf()
        } else {
            PathBuf::from(".")
        };
        #[cfg(not(test))]
        let current_dir = PathBuf::from(".");

        let output = Command::new("git")
            .current_dir(&current_dir)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute git command: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git command failed: {}", error));
        }

        Ok(())
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to get current branch: {}", e))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get current branch"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

    #[test]
    fn test_cli_interface() {
        let mock_provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(mock_provider));
        cli.init_test_repo();

        // Test team management
        cli.set_current_team("test-team".to_string());
        assert_eq!(cli.current_team, Some("test-team".to_string()));
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));

        // Test task management
        cli.add_task("test-task".to_string());
        assert_eq!(cli.current_task, Some("test-task".to_string()));
        assert_eq!(cli.current_branch, Some("feature/test-team-test-task".to_string()));

        // Test task completion
        cli.complete_task();
        assert_eq!(cli.current_task, None);
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));

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

    #[test]
    fn test_git_workflow() {
        let mock_provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(mock_provider));
        cli.init_test_repo();

        // Test team branch creation
        cli.set_current_team("test-team".to_string());
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));

        // Test task branch creation
        cli.add_task("test-task".to_string());
        assert_eq!(cli.current_branch, Some("feature/test-team-test-task".to_string()));

        // Test task completion
        cli.complete_task();
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));
    }

    #[test]
    fn test_git_branch_management() {
        let mock_provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(mock_provider));
        cli.init_test_repo();

        // Test creating multiple teams
        cli.set_current_team("team1".to_string());
        assert_eq!(cli.current_branch, Some("master-team1".to_string()));

        cli.set_current_team("team2".to_string());
        assert_eq!(cli.current_branch, Some("master-team2".to_string()));

        // Test creating multiple tasks
        cli.add_task("task1".to_string());
        assert_eq!(cli.current_branch, Some("feature/team2-task1".to_string()));

        cli.add_task("task2".to_string());
        assert_eq!(cli.current_branch, Some("feature/team2-task2".to_string()));

        // Test completing tasks
        cli.complete_task();
        assert_eq!(cli.current_branch, Some("master-team2".to_string()));
        assert_eq!(cli.current_task, None);
    }

    #[test]
    fn test_git_error_handling() {
        let mock_provider = MockLLMProvider::new();
        let mut cli = CliInterface::new(Box::new(mock_provider));
        cli.init_test_repo();

        // Test handling of invalid team names
        cli.set_current_team("test team".to_string());
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));

        // Test handling of invalid task names
        cli.add_task("test task".to_string());
        assert_eq!(cli.current_branch, Some("feature/test-team-test-task".to_string()));

        // Test completing task without current task
        cli.complete_task();
        assert_eq!(cli.current_task, None);
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));

        // Test completing task again (should be no-op)
        cli.complete_task();
        assert_eq!(cli.current_task, None);
        assert_eq!(cli.current_branch, Some("master-test-team".to_string()));
    }
} 