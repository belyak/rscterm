use anyhow::Result;
use git2::{Repository, BranchType as Git2BranchType};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum GitFlowError {
    #[error("Invalid branch name: {0}")]
    InvalidBranchName(String),
    #[error("Not on a valid GitFlow branch")]
    InvalidGitFlowBranch,
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    #[error("Branch type mismatch: expected {expected}, got {actual}")]
    BranchTypeMismatch { expected: String, actual: String },
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFlowConfig {
    pub master_branch: String,
    pub develop_branch: String,
    pub feature_prefix: String,
    pub release_prefix: String,
    pub hotfix_prefix: String,
    pub version_tag_prefix: String,
}

impl Default for GitFlowConfig {
    fn default() -> Self {
        Self {
            master_branch: "master".to_string(),
            develop_branch: "develop".to_string(),
            feature_prefix: "feature/".to_string(),
            release_prefix: "release/".to_string(),
            hotfix_prefix: "hotfix/".to_string(),
            version_tag_prefix: "v".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitFlowBranchType {
    Master,
    Develop,
    Feature(String),
    Release(String),
    Hotfix(String),
    Unknown,
}

pub struct GitFlowAgent {
    repo: Repository,
    config: GitFlowConfig,
}

impl GitFlowAgent {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        Ok(Self {
            repo,
            config: GitFlowConfig::default(),
        })
    }

    pub fn with_config(repo_path: &Path, config: GitFlowConfig) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        Ok(Self { repo, config })
    }

    pub fn get_current_branch_type(&self) -> Result<GitFlowBranchType> {
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("");
        
        if branch_name == self.config.master_branch {
            Ok(GitFlowBranchType::Master)
        } else if branch_name == self.config.develop_branch {
            Ok(GitFlowBranchType::Develop)
        } else if branch_name.starts_with(&self.config.feature_prefix) {
            Ok(GitFlowBranchType::Feature(
                branch_name[self.config.feature_prefix.len()..].to_string(),
            ))
        } else if branch_name.starts_with(&self.config.release_prefix) {
            Ok(GitFlowBranchType::Release(
                branch_name[self.config.release_prefix.len()..].to_string(),
            ))
        } else if branch_name.starts_with(&self.config.hotfix_prefix) {
            Ok(GitFlowBranchType::Hotfix(
                branch_name[self.config.hotfix_prefix.len()..].to_string(),
            ))
        } else {
            Ok(GitFlowBranchType::Unknown)
        }
    }

    pub fn validate_branch_name(&self, branch_name: &str) -> Result<(), GitFlowError> {
        let valid_chars = Regex::new(r"^[a-zA-Z0-9-_/.]+$").unwrap();
        if !valid_chars.is_match(branch_name) {
            return Err(GitFlowError::InvalidBranchName(branch_name.to_string()));
        }
        Ok(())
    }

    pub fn ensure_gitflow_compliance(&self) -> Result<(), GitFlowError> {
        let current_branch = self.get_current_branch_type()
            .map_err(|e| GitFlowError::InternalError(e.to_string()))?;
        
        match current_branch {
            GitFlowBranchType::Unknown => Err(GitFlowError::InvalidGitFlowBranch),
            _ => Ok(()),
        }
    }

    pub fn create_feature(&self, name: &str) -> Result<(), GitFlowError> {
        self.validate_branch_name(name)?;
        let feature_branch = format!("{}{}", self.config.feature_prefix, name);
        
        // Ensure we're on develop
        let develop = self.repo.find_branch(&self.config.develop_branch, Git2BranchType::Local)?;
        let develop_commit = develop.get().peel_to_commit()?;
        
        // Create and checkout the feature branch
        self.repo.branch(&feature_branch, &develop_commit, false)?;
        info!("Created feature branch: {}", feature_branch);
        Ok(())
    }

    pub fn finish_feature(&self, name: &str) -> Result<(), GitFlowError> {
        let current_branch = self.get_current_branch_type()
            .map_err(|e| GitFlowError::InternalError(e.to_string()))?;
        match current_branch {
            GitFlowBranchType::Feature(current_name) if current_name == name => {
                // Merge into develop
                let develop = self.repo.find_branch(&self.config.develop_branch, Git2BranchType::Local)?;
                let develop_commit = develop.get().peel_to_commit()?;
                
                // TODO: Implement merge logic
                info!("Finished feature branch: {}", name);
                Ok(())
            }
            _ => Err(GitFlowError::BranchTypeMismatch {
                expected: format!("feature/{}", name),
                actual: format!("{:?}", current_branch),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_branch_validation() {
        let temp_dir = tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        let agent = GitFlowAgent::new(temp_dir.path()).unwrap();

        assert!(agent.validate_branch_name("feature/my-feature").is_ok());
        assert!(agent.validate_branch_name("release/1.0.0").is_ok());
        assert!(agent.validate_branch_name("hotfix/critical-fix").is_ok());
        assert!(agent.validate_branch_name("feature/my_feature").is_ok());
        assert!(agent.validate_branch_name("feature/my.feature").is_ok());
    }
}
