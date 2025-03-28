use rscterm_core::error::Result;

pub struct AIbitatHelper {
    commands: Vec<String>,
    team_suggestions: Vec<String>,
    task_suggestions: Vec<String>,
    model_suggestions: Vec<String>,
}

impl AIbitatHelper {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "help".to_string(),
                "exit".to_string(),
                "clear".to_string(),
                "history".to_string(),
                "send".to_string(),
                "list".to_string(),
                "agents".to_string(),
                "channels".to_string(),
                "setup-team".to_string(),
                "start-task".to_string(),
                "toggle-progress".to_string(),
                "chat".to_string(),
                "list-models".to_string(),
                "set-model".to_string(),
            ],
            team_suggestions: vec![
                "research-team".to_string(),
                "development-team".to_string(),
                "design-team".to_string(),
                "product-team".to_string(),
                "qa-team".to_string(),
            ],
            task_suggestions: vec![
                "research-market-trends".to_string(),
                "develop-new-feature".to_string(),
                "design-user-interface".to_string(),
                "review-code-changes".to_string(),
                "test-functionality".to_string(),
            ],
            model_suggestions: vec![
                "gemma-3-12b-it".to_string(),
                "gemma-2b-it".to_string(),
                "mistral-7b".to_string(),
                "llama-2-7b".to_string(),
                "codellama-7b".to_string(),
            ],
        }
    }

    pub fn get_completions(&self, input: &str) -> Vec<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return self.commands.clone();
        }

        let cmd = parts[0];
        match cmd {
            "setup-team" => {
                if parts.len() == 1 {
                    return self.team_suggestions.clone();
                }
                let partial = parts[1];
                self.team_suggestions
                    .iter()
                    .filter(|s| s.starts_with(partial))
                    .cloned()
                    .collect()
            }
            "start-task" => {
                if parts.len() == 1 {
                    return self.task_suggestions.clone();
                }
                let partial = parts[1];
                self.task_suggestions
                    .iter()
                    .filter(|s| s.starts_with(partial))
                    .cloned()
                    .collect()
            }
            "set-model" => {
                if parts.len() == 1 {
                    return self.model_suggestions.clone();
                }
                let partial = parts[1];
                self.model_suggestions
                    .iter()
                    .filter(|s| s.starts_with(partial))
                    .cloned()
                    .collect()
            }
            _ => self
                .commands
                .iter()
                .filter(|s| s.starts_with(cmd))
                .cloned()
                .collect(),
        }
    }

    pub fn get_hint(&self, input: &str) -> Option<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Some("Type a command and press TAB for suggestions".to_string());
        }

        let cmd = parts[0];
        match cmd {
            "setup-team" => Some("Create a new team with agents (e.g., setup-team research-team researcher programmer)".to_string()),
            "start-task" => Some("Start a new task (e.g., start-task research-market-trends)".to_string()),
            "set-model" => Some("Change the current model (e.g., set-model gemma-3-12b-it)".to_string()),
            "send" => Some("Send a message to a channel (e.g., send #general Hello world)".to_string()),
            "chat" => Some("Start a direct chat with the AI (e.g., chat What is the best way to structure a web application?)".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_creation() {
        let helper = AIbitatHelper::new();
        assert!(!helper.commands.is_empty());
        assert!(!helper.team_suggestions.is_empty());
        assert!(!helper.task_suggestions.is_empty());
        assert!(!helper.model_suggestions.is_empty());
    }

    #[test]
    fn test_command_completion() {
        let helper = AIbitatHelper::new();
        let completions = helper.get_completions("");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"help".to_string()));
        assert!(completions.contains(&"exit".to_string()));
    }

    #[test]
    fn test_team_completion() {
        let helper = AIbitatHelper::new();
        let completions = helper.get_completions("setup-team");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"research-team".to_string()));
        assert!(completions.contains(&"development-team".to_string()));
    }

    #[test]
    fn test_task_completion() {
        let helper = AIbitatHelper::new();
        let completions = helper.get_completions("start-task");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"research-market-trends".to_string()));
        assert!(completions.contains(&"develop-new-feature".to_string()));
    }

    #[test]
    fn test_model_completion() {
        let helper = AIbitatHelper::new();
        let completions = helper.get_completions("set-model");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"gemma-3-12b-it".to_string()));
        assert!(completions.contains(&"mistral-7b".to_string()));
    }

    #[test]
    fn test_partial_completion() {
        let helper = AIbitatHelper::new();
        let completions = helper.get_completions("setup-team res");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"research-team".to_string()));
    }

    #[test]
    fn test_hints() {
        let helper = AIbitatHelper::new();
        assert!(helper.get_hint("setup-team").is_some());
        assert!(helper.get_hint("start-task").is_some());
        assert!(helper.get_hint("set-model").is_some());
        assert!(helper.get_hint("send").is_some());
        assert!(helper.get_hint("chat").is_some());
        assert!(helper.get_hint("unknown").is_none());
    }
} 