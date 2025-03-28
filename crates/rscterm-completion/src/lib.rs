use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct CompletionState {
    teams: HashSet<String>,
    tasks: HashSet<String>,
    models: HashSet<String>,
}

pub struct AIbitatHelper {
    pub highlighter: MatchingBracketHighlighter,
    pub hinter: HistoryHinter,
    state: Arc<Mutex<CompletionState>>,
}

impl AIbitatHelper {
    pub fn new() -> Self {
        Self {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            state: Arc::new(Mutex::new(CompletionState::default())),
        }
    }

    pub fn add_team(&mut self, team: String) {
        if let Ok(mut state) = self.state.lock() {
            state.teams.insert(team);
        }
    }

    pub fn add_task(&mut self, task: String) {
        if let Ok(mut state) = self.state.lock() {
            state.tasks.insert(task);
        }
    }

    pub fn update_models(&mut self, models: Vec<String>) {
        if let Ok(mut state) = self.state.lock() {
            state.models = models.into_iter().collect();
        }
    }

    fn get_base_commands() -> Vec<&'static str> {
        vec![
            "help",
            "clear",
            "setup-team",
            "start-task",
            "chat",
            "exit",
            "list-models",
            "set-model",
            "toggle-progress",
        ]
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
        let state = self.state.lock().unwrap();

        match line_parts.get(0) {
            Some(&"setup-team") => {
                if line_parts.len() <= 2 {
                    completions.extend(
                        state.teams
                            .iter()
                            .filter(|team| current_word.is_empty() || team.starts_with(current_word))
                            .map(|team| Pair {
                                display: team.clone(),
                                replacement: team.clone(),
                            }),
                    );
                }
            }
            Some(&"start-task") => {
                if line_parts.len() <= 2 {
                    completions.extend(
                        state.tasks
                            .iter()
                            .filter(|task| current_word.is_empty() || task.starts_with(current_word))
                            .map(|task| Pair {
                                display: task.clone(),
                                replacement: task.clone(),
                            }),
                    );
                }
            }
            Some(&"set-model") => {
                if line_parts.len() <= 2 {
                    completions.extend(
                        state.models
                            .iter()
                            .filter(|model| current_word.is_empty() || model.starts_with(current_word))
                            .map(|model| Pair {
                                display: model.clone(),
                                replacement: model.clone(),
                            }),
                    );
                }
            }
            Some(cmd) if line_parts.len() == 1 && !line.ends_with(' ') => {
                completions.extend(
                    Self::get_base_commands()
                        .into_iter()
                        .filter(|c| c.starts_with(cmd))
                        .map(|c| Pair {
                            display: c.to_string(),
                            replacement: c.to_string(),
                        }),
                );
            }
            None => {
                completions.extend(
                    Self::get_base_commands()
                        .into_iter()
                        .filter(|cmd| cmd.starts_with(current_word))
                        .map(|cmd| Pair {
                            display: cmd.to_string(),
                            replacement: cmd.to_string(),
                        }),
                );
            }
            _ => {}
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
            l if l.starts_with("setup-team") => Some(" Create a new team with agents".to_string()),
            l if l.starts_with("start-task") => Some(" Start a new task".to_string()),
            l if l.starts_with("chat") => Some(" Type your message".to_string()),
            l if l.starts_with("set-model") => Some(" Change the current model".to_string()),
            l if l.starts_with("list-models") => Some(" Shows available models".to_string()),
            l if l.starts_with("toggle-progress") => Some(" Toggle progress bar visibility".to_string()),
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
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_helper_creation() {
        let helper = AIbitatHelper::new();
        let state = helper.state.lock().unwrap();
        assert!(state.teams.is_empty());
        assert!(state.tasks.is_empty());
        assert!(state.models.is_empty());
    }

    #[test]
    fn test_add_team() {
        let mut helper = AIbitatHelper::new();
        helper.add_team("test-team".to_string());
        let state = helper.state.lock().unwrap();
        assert!(state.teams.contains("test-team"));
    }

    #[test]
    fn test_add_task() {
        let mut helper = AIbitatHelper::new();
        helper.add_task("test-task".to_string());
        let state = helper.state.lock().unwrap();
        assert!(state.tasks.contains("test-task"));
    }

    #[test]
    fn test_update_models() {
        let mut helper = AIbitatHelper::new();
        helper.update_models(vec!["model1".to_string(), "model2".to_string()]);
        let state = helper.state.lock().unwrap();
        assert!(state.models.contains("model1"));
        assert!(state.models.contains("model2"));
    }

    #[test]
    fn test_command_completion() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        // Test base command completion
        let (pos, completions) = helper.complete("he", 2, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(completions.iter().any(|c| c.replacement == "help"));

        // Test empty completion
        let (pos, completions) = helper.complete("", 0, &ctx).unwrap();
        assert_eq!(pos, 0);
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_hints() {
        let helper = AIbitatHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        assert!(helper.hint("setup-team", 0, &ctx).is_some());
        assert!(helper.hint("start-task", 0, &ctx).is_some());
        assert!(helper.hint("chat", 0, &ctx).is_some());
        assert!(helper.hint("set-model", 0, &ctx).is_some());
        assert!(helper.hint("list-models", 0, &ctx).is_some());
        assert!(helper.hint("toggle-progress", 0, &ctx).is_some());
        assert!(helper.hint("unknown", 0, &ctx).is_none());
    }
}
