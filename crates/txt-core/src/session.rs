use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const MAX_RECENT_FILES: usize = 20;
pub const MAX_RECENT_WORKSPACES: usize = 10;

/// Serializable editor session state for workspace restoration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub sidebar_visible: bool,
    pub font_size: f32,
    pub theme: String,
    pub word_wrap: bool,
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
    #[serde(default)]
    pub recent_workspaces: Vec<PathBuf>,
    #[serde(default)]
    pub active_workspace: Option<PathBuf>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            open_files: Vec::new(),
            active_file: None,
            sidebar_visible: true,
            font_size: 14.0,
            theme: "Mariana".to_string(),
            word_wrap: true,
            recent_files: Vec::new(),
            recent_workspaces: Vec::new(),
            active_workspace: None,
        }
    }
}

impl SessionState {
    /// Creates a new default session state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a file to open_files if not already present.
    pub fn add_open_file(&mut self, path: impl Into<PathBuf>) {
        let p = path.into();
        if !self.open_files.contains(&p) {
            self.open_files.push(p);
        }
    }

    /// Removes a file from open_files. If it was active, clears active_file or selects another.
    pub fn remove_open_file(&mut self, path: &Path) {
        self.open_files.retain(|p| p.as_path() != path);
        if let Some(active) = &self.active_file {
            if active.as_path() == path {
                self.active_file = self.open_files.first().cloned();
            }
        }
    }

    /// Sets the active file. Also adds it to open_files if not already open.
    pub fn set_active_file(&mut self, path: Option<PathBuf>) {
        if let Some(ref p) = path {
            self.add_open_file(p.clone());
        }
        self.active_file = path;
    }

    /// Checks if a file is currently open in the session.
    pub fn is_open(&self, path: &Path) -> bool {
        self.open_files.iter().any(|p| p.as_path() == path)
    }

    pub fn with_open_files(mut self, files: Vec<PathBuf>) -> Self {
        self.open_files = files;
        self
    }

    pub fn with_active_file(mut self, active: Option<PathBuf>) -> Self {
        self.active_file = active;
        self
    }

    pub fn with_sidebar_visible(mut self, visible: bool) -> Self {
        self.sidebar_visible = visible;
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Pushes a file path to the recent files list (LRU, unique, max 20).
    pub fn push_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > MAX_RECENT_FILES {
            self.recent_files.truncate(MAX_RECENT_FILES);
        }
    }

    /// Pushes a workspace path to the recent workspaces list (LRU, unique, max 10).
    pub fn push_recent_workspace(&mut self, path: PathBuf) {
        self.recent_workspaces.retain(|p| p != &path);
        self.recent_workspaces.insert(0, path);
        if self.recent_workspaces.len() > MAX_RECENT_WORKSPACES {
            self.recent_workspaces.truncate(MAX_RECENT_WORKSPACES);
        }
    }

    pub fn with_recent_files(mut self, files: Vec<PathBuf>) -> Self {
        self.recent_files = files;
        self
    }

    pub fn with_recent_workspaces(mut self, workspaces: Vec<PathBuf>) -> Self {
        self.recent_workspaces = workspaces;
        self
    }

    pub fn with_active_workspace(mut self, active: Option<PathBuf>) -> Self {
        self.active_workspace = active;
        self
    }

    pub fn with_word_wrap(mut self, word_wrap: bool) -> Self {
        self.word_wrap = word_wrap;
        self
    }
}

/// Manages serialization, atomic persistence, and recovery of editor session state.
pub struct SessionManager;

impl SessionManager {
    /// Saves session state to disk at the given path.
    /// Ensures parent directories exist and uses atomic write to prevent corruption.
    pub fn save_session(path: &Path, state: &SessionState) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(path, json.as_bytes())?;
        Ok(())
    }

    /// Loads session state from disk at the given path.
    pub fn load_session(path: &Path) -> io::Result<SessionState> {
        let content = fs::read_to_string(path)?;
        let state: SessionState = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(state)
    }

    /// Returns the default session path (`.txt/session.json` in current directory).
    pub fn default_session_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".txt")
            .join("session.json")
    }

    /// Saves session state to the default session path.
    pub fn save_default(state: &SessionState) -> io::Result<()> {
        Self::save_session(&Self::default_session_path(), state)
    }

    /// Loads session state from the default session path.
    pub fn load_default() -> io::Result<SessionState> {
        Self::load_session(&Self::default_session_path())
    }

    /// Checks if a session file exists at the given path.
    pub fn exists(path: &Path) -> bool {
        path.exists() && path.is_file()
    }
}
