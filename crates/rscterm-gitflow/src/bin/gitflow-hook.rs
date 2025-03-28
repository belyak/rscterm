use anyhow::Result;
use rscterm_gitflow::{GitFlowAgent, GitFlowError};
use std::env;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

fn setup_logging() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .init();
}

fn main() -> Result<()> {
    setup_logging();

    let current_dir = env::current_dir()?;
    let repo_path = find_git_root(&current_dir)?;
    
    let agent = GitFlowAgent::new(&repo_path)?;
    
    match agent.ensure_gitflow_compliance() {
        Ok(_) => {
            info!("GitFlow compliance check passed");
            Ok(())
        }
        Err(GitFlowError::InvalidGitFlowBranch) => {
            error!("Not on a valid GitFlow branch. Please use feature/, release/, or hotfix/ branches");
            std::process::exit(1);
        }
        Err(e) => {
            error!("GitFlow check failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn find_git_root(start_path: &PathBuf) -> Result<PathBuf> {
    let mut current_path = start_path.clone();
    loop {
        let git_dir = current_path.join(".git");
        if git_dir.exists() {
            return Ok(current_path);
        }
        
        if !current_path.pop() {
            anyhow::bail!("Could not find Git repository root");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_git_root() {
        let current_dir = env::current_dir().unwrap();
        let result = find_git_root(&current_dir);
        assert!(result.is_ok());
    }
} 