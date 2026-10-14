use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::fuzzy::{FuzzyMatch, FuzzyMatcher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandDescriptor {
    pub id: String,
    pub title: String,
    pub category: String,
    pub shortcut: Option<String>,
    pub description: Option<String>,
}

impl CommandDescriptor {
    pub fn new(id: impl Into<String>, title: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: category.into(),
            shortcut: None,
            description: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn full_display_name(&self) -> String {
        format!("{}: {}", self.category, self.title)
    }
}

#[derive(Clone)]
pub struct Command {
    pub descriptor: CommandDescriptor,
    pub handler: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("descriptor", &self.descriptor)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct MatchedCommand {
    pub descriptor: CommandDescriptor,
    pub match_result: FuzzyMatch,
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn register<F>(&mut self, descriptor: CommandDescriptor, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.commands.push(Command {
            descriptor,
            handler: Arc::new(handler),
        });
    }

    pub fn list(&self) -> Vec<CommandDescriptor> {
        self.commands.iter().map(|c| c.descriptor.clone()).collect()
    }

    /// Fuzzy searches all registered commands, ranking by Sublime Text score.
    pub fn search(&self, query: &str) -> Vec<MatchedCommand> {
        let mut results = Vec::new();

        for cmd in &self.commands {
            let target = cmd.descriptor.full_display_name();
            if let Some(match_result) = FuzzyMatcher::match_str(query, &target) {
                results.push(MatchedCommand {
                    descriptor: cmd.descriptor.clone(),
                    match_result,
                });
            }
        }

        // Sort descending by match score
        results.sort_by(|a, b| b.match_result.score.cmp(&a.match_result.score));
        results
    }

    /// Executes command by ID.
    pub fn execute(&self, id: &str) -> bool {
        if let Some(cmd) = self.commands.iter().find(|c| c.descriptor.id == id) {
            (cmd.handler)();
            true
        } else {
            false
        }
    }
}
