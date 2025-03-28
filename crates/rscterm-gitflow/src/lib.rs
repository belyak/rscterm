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
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
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
    pub fn new(repo_path: &Path) -> Result<Self, GitFlowError> {
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
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("");
        
        if !is_gitflow_branch(branch_name) {
            return Err(GitFlowError::InvalidGitFlowBranch);
        }
        
        Ok(())
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
                let _develop_commit = develop.get().peel_to_commit()?;
                
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

fn is_gitflow_branch(branch: &str) -> bool {
    let valid_prefixes = ["feature/", "bugfix/", "hotfix/", "release/"];
    valid_prefixes.iter().any(|prefix| branch.starts_with(prefix))
}

#[allow(dead_code)]
fn validate_branch_name(branch: &str) -> bool {
    // Check if it's a valid GitFlow branch
    if !is_gitflow_branch(branch) {
        return false;
    }

    // Check for invalid characters
    if branch.contains(' ') || branch.contains('.') || branch.contains('_') {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_repo() -> (tempfile::TempDir, Repository) {
        let temp_dir = tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();
        
        // Create initial commit
        let mut index = repo.index().unwrap();
        index.add_all(["."], git2::IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        
        let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();
        
        // Create develop branch
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("develop", &head, false).unwrap();
        
        (temp_dir, repo)
    }

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
        assert!(agent.validate_branch_name("feature/my@feature").is_err());
    }

    #[test]
    fn test_branch_type_detection() {
        let (_temp_dir, repo) = setup_test_repo();
        let agent = GitFlowAgent::new(repo.path()).unwrap();

        // Test master branch
        repo.set_head("refs/heads/master").unwrap();
        assert!(matches!(agent.get_current_branch_type().unwrap(), GitFlowBranchType::Master));

        // Test develop branch
        repo.set_head("refs/heads/develop").unwrap();
        assert!(matches!(agent.get_current_branch_type().unwrap(), GitFlowBranchType::Develop));

        // Test feature branch
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature/test-feature", &head, false).unwrap();
        repo.set_head("refs/heads/feature/test-feature").unwrap();
        if let GitFlowBranchType::Feature(name) = agent.get_current_branch_type().unwrap() {
            assert_eq!(name, "test-feature");
        } else {
            panic!("Expected Feature branch type");
        }

        // Test release branch
        repo.branch("release/1.0.0", &head, false).unwrap();
        repo.set_head("refs/heads/release/1.0.0").unwrap();
        if let GitFlowBranchType::Release(version) = agent.get_current_branch_type().unwrap() {
            assert_eq!(version, "1.0.0");
        } else {
            panic!("Expected Release branch type");
        }

        // Test hotfix branch
        repo.branch("hotfix/critical-fix", &head, false).unwrap();
        repo.set_head("refs/heads/hotfix/critical-fix").unwrap();
        if let GitFlowBranchType::Hotfix(name) = agent.get_current_branch_type().unwrap() {
            assert_eq!(name, "critical-fix");
        } else {
            panic!("Expected Hotfix branch type");
        }

        // Test unknown branch
        repo.branch("random-branch", &head, false).unwrap();
        repo.set_head("refs/heads/random-branch").unwrap();
        assert!(matches!(agent.get_current_branch_type().unwrap(), GitFlowBranchType::Unknown));
    }

    #[test]
    fn test_gitflow_compliance() {
        let (_temp_dir, repo) = setup_test_repo();
        let agent = GitFlowAgent::new(repo.path()).unwrap();

        // Test valid branches
        repo.set_head("refs/heads/master").unwrap();
        assert!(agent.ensure_gitflow_compliance().is_ok());

        repo.set_head("refs/heads/develop").unwrap();
        assert!(agent.ensure_gitflow_compliance().is_ok());

        // Test invalid branch
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("random-branch", &head, false).unwrap();
        repo.set_head("refs/heads/random-branch").unwrap();
        assert!(agent.ensure_gitflow_compliance().is_err());
    }

    #[test]
    fn test_feature_branch_operations() {
        let (_temp_dir, repo) = setup_test_repo();
        let agent = GitFlowAgent::new(repo.path()).unwrap();

        // Test creating feature branch
        repo.set_head("refs/heads/develop").unwrap();
        assert!(agent.create_feature("test-feature").is_ok());
        
        // Verify branch was created
        assert!(repo.find_branch("feature/test-feature", Git2BranchType::Local).is_ok());

        // Test finishing feature branch
        repo.set_head("refs/heads/feature/test-feature").unwrap();
        assert!(agent.finish_feature("test-feature").is_ok());

        // Test finishing feature from wrong branch
        repo.set_head("refs/heads/develop").unwrap();
        assert!(agent.finish_feature("test-feature").is_err());
    }

    #[test]
    fn test_is_gitflow_branch() {
        assert!(is_gitflow_branch("feature/test"));
        assert!(is_gitflow_branch("bugfix/test"));
        assert!(is_gitflow_branch("hotfix/test"));
        assert!(is_gitflow_branch("release/test"));
        assert!(!is_gitflow_branch("main"));
        assert!(!is_gitflow_branch("develop"));
        assert!(!is_gitflow_branch("test"));
    }

    #[test]
    fn test_validate_branch_name() {
        assert!(validate_branch_name("feature/test-branch"));
        assert!(validate_branch_name("bugfix/issue-123"));
        assert!(validate_branch_name("hotfix/critical-fix"));
        assert!(validate_branch_name("release/v1.0.0"));
        assert!(!validate_branch_name("feature/test branch")); // Contains space
        assert!(!validate_branch_name("feature/test.branch")); // Contains dot
        assert!(!validate_branch_name("feature/test_branch")); // Contains underscore
        assert!(!validate_branch_name("invalid/test")); // Invalid prefix
    }

    #[test]
    fn test_gitflow_agent() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        env::set_current_dir(dir.path()).unwrap();

        // Initialize git repo
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Create a GitFlow agent
        let agent = GitFlowAgent::new(dir.path()).unwrap();

        // Test with invalid branch
        std::process::Command::new("git")
            .args(&["checkout", "-b", "main"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(agent.ensure_gitflow_compliance().is_err());

        // Test with valid branch
        std::process::Command::new("git")
            .args(&["checkout", "-b", "feature/test"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(agent.ensure_gitflow_compliance().is_ok());
    }
}
