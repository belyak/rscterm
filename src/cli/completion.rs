use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::collections::HashSet;

#[derive(Default)]
pub struct AIbitatHelper {
    pub highlighter: MatchingBracketHighlighter,
    pub hinter: HistoryHinter,
    teams: HashSet<String>,
}

impl AIbitatHelper {
    pub fn new() -> Self {
        Self {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            teams: HashSet::new(),
        }
    }

    pub fn add_team(&mut self, team: String) {
        self.teams.insert(team);
    }
}

impl Completer for AIbitatHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let commands = vec![
            "help",
            "clear",
            "setup-team",
            "start-task",
            "chat",
            "exit",
            "list-models",
            "set-model",
        ];

        let line_parts: Vec<&str> = line[..pos].split_whitespace().collect();
        let current_word = if line[..pos].ends_with(' ') {
            ""
        } else {
            line_parts.last().unwrap_or(&"")
        };

        let start_pos = if current_word.is_empty() {
            pos
        } else {
            pos - current_word.len()
        };

        let mut completions = Vec::new();

        match line_parts.get(0) {
            Some(&"setup-team") => {
                if line_parts.len() <= 2 {
                    // Provide default team suggestions
                    completions.extend(vec![
                        "research-team",
                        "development-team",
                        "design-team",
                        "product-team",
                        "qa-team"
                    ].iter().filter(|team| {
                        current_word.is_empty() || team.starts_with(current_word)
                    }).map(|team| Pair {
                        display: (*team).to_string(),
                        replacement: (*team).to_string(),
                    }));
                }
            }
            Some(&"start-task") => {
                if line_parts.len() <= 2 {
                    // Provide default task suggestions
                    completions.extend(vec![
                        "research-market-trends",
                        "develop-new-feature",
                        "design-user-interface",
                        "review-code-changes",
                        "test-functionality",
                        "develop-sokoban-game",
                        "implement-game-logic",
                        "create-game-assets",
                        "optimize-game-performance",
                        "add-game-features"
                    ].iter().filter(|task| {
                        current_word.is_empty() || task.starts_with(current_word)
                    }).map(|task| Pair {
                        display: (*task).to_string(),
                        replacement: (*task).to_string(),
                    }));
                }
            }
            Some(&"chat") => {
                if line_parts.len() <= 2 {
                    // No suggestions for chat messages, as they are user-defined
                }
            }
            Some(&"set-model") => {
                if line_parts.len() <= 2 {
                    // Provide default model suggestions
                    completions.extend(vec![
                        "gemma-3-12b-it",
                        "gemma-2b-it",
                        "mistral-7b",
                        "llama-2-7b",
                        "codellama-7b"
                    ].iter().filter(|model| {
                        current_word.is_empty() || model.starts_with(current_word)
                    }).map(|model| Pair {
                        display: (*model).to_string(),
                        replacement: (*model).to_string(),
                    }));
                }
            }
            Some(cmd) if line_parts.len() == 1 && !line.ends_with(' ') => {
                // Complete partial command
                completions.extend(
                    commands
                        .iter()
                        .filter(|c| c.starts_with(cmd))
                        .map(|c| Pair {
                            display: (*c).to_string(),
                            replacement: (*c).to_string(),
                        }),
                );
            }
            Some(_) => {
                // No completions for unknown commands or commands with arguments
            }
            None => {
                // Complete base commands only when at start of line
                completions.extend(
                    commands
                        .iter()
                        .filter(|cmd| cmd.starts_with(current_word))
                        .map(|cmd| Pair {
                            display: (*cmd).to_string(),
                            replacement: (*cmd).to_string(),
                        }),
                );
            }
        }

        Ok((start_pos, completions))
    }
}

impl Hinter for AIbitatHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        match line {
            l if l.starts_with("setup-team") => Some(" Examples: research-team, development-team, design-team".to_string()),
            l if l.starts_with("start-task") => Some(" Examples: develop-sokoban-game, implement-game-logic, create-game-assets".to_string()),
            l if l.starts_with("chat") => Some(" Type your message".to_string()),
            l if l.starts_with("set-model") => Some(" Examples: gemma-3-12b-it, mistral-7b, llama-2-7b".to_string()),
            l if l.starts_with("list-models") => Some(" Shows available models in LM Studio".to_string()),
            _ => None,
        }
    }
}

impl Highlighter for AIbitatHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        std::borrow::Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Borrowed(hint)
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for AIbitatHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for AIbitatHelper {}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::completion::Completer;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_helper_creation() {
        let helper = AIbitatHelper::new();
        assert!(helper.teams.is_empty());
    }

    #[test]
    fn test_add_team() {
        let mut helper = AIbitatHelper::new();
        helper.add_team("test-team".to_string());
        assert!(helper.teams.contains("test-team"));
    }

    #[test]
    fn test_command_completion() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test basic command completion
        let (pos, completions) = helper.complete("he", 2, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(completions.iter().any(|c| c.replacement == "help"));

        // Test empty completion
        let (pos, completions) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(!completions.is_empty()); // Should show all available commands
        assert!(completions.iter().any(|c| c.replacement == "help"));
        assert!(completions.iter().any(|c| c.replacement == "clear"));
        assert!(completions.iter().any(|c| c.replacement == "setup-team"));
        assert!(completions.iter().any(|c| c.replacement == "start-task"));
        assert!(completions.iter().any(|c| c.replacement == "chat"));
        assert!(completions.iter().any(|c| c.replacement == "exit"));
        assert!(completions.iter().any(|c| c.replacement == "list-models"));
        assert!(completions.iter().any(|c| c.replacement == "set-model"));

        // Test completion with space
        let (pos, completions) = helper.complete("help ", 5, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_setup_team_completion() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test setup-team command completion with empty input
        let (pos, completions) = helper.complete("setup-team ", 11, &ctx).unwrap();
        assert_eq!(pos, 11);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.replacement == "research-team"));
        assert!(completions.iter().any(|c| c.replacement == "development-team"));
        assert!(completions.iter().any(|c| c.replacement == "design-team"));
        assert!(completions.iter().any(|c| c.replacement == "product-team"));
        assert!(completions.iter().any(|c| c.replacement == "qa-team"));

        // Test setup-team command completion with partial input
        let (pos, completions) = helper.complete("setup-team dev", 14, &ctx).unwrap();
        assert_eq!(pos, 11);
        assert_eq!(completions.len(), 1);
        assert!(completions.iter().any(|c| c.replacement == "development-team"));
    }

    #[test]
    fn test_start_task_completion() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test start-task command completion with empty input
        let (pos, completions) = helper.complete("start-task ", 11, &ctx).unwrap();
        assert_eq!(pos, 11);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.replacement == "develop-sokoban-game"));
        assert!(completions.iter().any(|c| c.replacement == "implement-game-logic"));
        assert!(completions.iter().any(|c| c.replacement == "create-game-assets"));
        assert!(completions.iter().any(|c| c.replacement == "optimize-game-performance"));
        assert!(completions.iter().any(|c| c.replacement == "add-game-features"));

        // Test start-task command completion with partial input
        let (pos, completions) = helper.complete("start-task dev", 14, &ctx).unwrap();
        assert_eq!(pos, 11);
        assert!(completions.iter().any(|c| c.replacement == "develop-sokoban-game"));
    }

    #[test]
    fn test_chat_completion() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test chat command completion
        let (pos, completions) = helper.complete("chat ", 5, &ctx).unwrap();
        assert_eq!(pos, 5);
        assert!(completions.is_empty()); // No suggestions for chat messages
    }

    #[test]
    fn test_model_commands() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test set-model command completion with empty input
        let (pos, completions) = helper.complete("set-model ", 10, &ctx).unwrap();
        assert_eq!(pos, 10);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.replacement == "gemma-3-12b-it"));
        assert!(completions.iter().any(|c| c.replacement == "mistral-7b"));
        assert!(completions.iter().any(|c| c.replacement == "llama-2-7b"));

        // Test set-model command completion with partial input
        let (pos, completions) = helper.complete("set-model gem", 13, &ctx).unwrap();
        assert_eq!(pos, 10);
        assert!(completions.iter().any(|c| c.replacement == "gemma-3-12b-it"));
        assert!(completions.iter().any(|c| c.replacement == "gemma-2b-it"));

        // Test list-models command completion
        let (pos, completions) = helper.complete("list-models ", 12, &ctx).unwrap();
        assert_eq!(pos, 12);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_hints() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test setup-team hint
        let hint = helper.hint("setup-team", 0, &ctx);
        assert_eq!(hint, Some(" Examples: research-team, development-team, design-team".to_string()));

        // Test start-task hint
        let hint = helper.hint("start-task", 0, &ctx);
        assert_eq!(hint, Some(" Examples: develop-sokoban-game, implement-game-logic, create-game-assets".to_string()));

        // Test chat hint
        let hint = helper.hint("chat", 0, &ctx);
        assert_eq!(hint, Some(" Type your message".to_string()));

        // Test set-model hint
        let hint = helper.hint("set-model", 0, &ctx);
        assert_eq!(hint, Some(" Examples: gemma-3-12b-it, mistral-7b, llama-2-7b".to_string()));

        // Test list-models hint
        let hint = helper.hint("list-models", 0, &ctx);
        assert_eq!(hint, Some(" Shows available models in LM Studio".to_string()));

        // Test empty hint
        let hint = helper.hint("", 0, &ctx);
        assert!(hint.is_none());

        // Test unknown command hint
        let hint = helper.hint("unknown", 0, &ctx);
        assert!(hint.is_none());
    }
} 