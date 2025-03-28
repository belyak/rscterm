use anyhow::Result;
use rscterm_gitflow::GitFlowAgent;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use git2;

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

#[test]
fn test_find_git_root() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize git repository
    let repo = git2::Repository::init(repo_path).unwrap();
    
    // Create a subdirectory
    let subdir = repo_path.join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    // Test finding git root from subdirectory
    let found_root = find_git_root(&subdir).unwrap();
    assert_eq!(found_root, repo_path);
    
    // Test finding git root from root directory
    let found_root = find_git_root(repo_path).unwrap();
    assert_eq!(found_root, repo_path);
    
    // Test with non-git directory
    let non_git_dir = tempfile::tempdir().unwrap();
    assert!(find_git_root(non_git_dir.path()).is_err());
}

#[test]
fn test_gitflow_hook() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize git repository
    let repo = git2::Repository::init(repo_path).unwrap();
    
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
    
    // Set up environment for hook
    env::set_current_dir(repo_path).unwrap();
    
    // Test with valid branch
    repo.set_head("refs/heads/develop").unwrap();
    assert!(GitFlowAgent::new(repo_path).unwrap().ensure_gitflow_compliance().is_ok());
    
    // Test with invalid branch
    repo.branch("random-branch", &head, false).unwrap();
    repo.set_head("refs/heads/random-branch").unwrap();
    assert!(GitFlowAgent::new(repo_path).unwrap().ensure_gitflow_compliance().is_err());
} 