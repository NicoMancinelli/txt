use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Represents an individual folder mounted in the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl WorkspaceFolder {
    /// Creates a new workspace folder reference.
    pub fn new(path: impl Into<PathBuf>, name: Option<String>) -> Self {
        Self {
            path: path.into(),
            name,
        }
    }
}

/// Project-specific editor configuration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspaceSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_wrap: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
}

impl WorkspaceSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tab_size(mut self, size: usize) -> Self {
        self.tab_size = Some(size);
        self
    }

    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = Some(wrap);
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
}

/// Project / workspace configuration descriptor.
///
/// Fully serializable/deserializable with Serde and compatible with
/// Sublime Text `.sublime-project` files (`{"folders": [{"path": "..."}, ...]}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub folders: Vec<WorkspaceFolder>,
    #[serde(default)]
    pub settings: WorkspaceSettings,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

impl WorkspaceConfig {
    /// Creates a new empty workspace configuration with a project name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            folders: Vec::new(),
            settings: WorkspaceSettings::default(),
        }
    }

    /// Adds a folder to the workspace. If the folder path already exists,
    /// updates its display name if specified.
    pub fn add_folder(&mut self, path: PathBuf, name: Option<String>) {
        if let Some(folder) = self.folders.iter_mut().find(|f| f.path == path) {
            if name.is_some() {
                folder.name = name;
            }
        } else {
            self.folders.push(WorkspaceFolder::new(path, name));
        }
    }

    /// Removes a folder from the workspace by path.
    pub fn remove_folder(&mut self, path: &Path) {
        self.folders.retain(|f| f.path.as_path() != path);
    }

    /// Builder method to add a folder.
    pub fn with_folder(mut self, path: impl Into<PathBuf>, name: Option<String>) -> Self {
        self.add_folder(path.into(), name);
        self
    }

    /// Builder method to specify workspace settings.
    pub fn with_settings(mut self, settings: WorkspaceSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Saves the workspace configuration to disk in JSON format.
    /// Ensures parent directories exist.
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json.as_bytes())?;
        Ok(())
    }

    /// Loads workspace configuration from disk.
    /// Gracefully handles Sublime Text `.sublime-project` files or txt workspace JSON.
    /// If the configuration has an empty name, defaults to the file stem of `path`.
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut config: Self = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if config.name.is_empty() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                config.name = stem.to_string();
            } else {
                config.name = "Untitled".to_string();
            }
        }
        Ok(config)
    }

    /// Parses a workspace configuration from a Sublime Text `.sublime-project` JSON string.
    pub fn from_sublime_project_str(content: &str, fallback_name: Option<&str>) -> io::Result<Self> {
        let mut config: Self = serde_json::from_str(content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if config.name.is_empty() {
            config.name = fallback_name.unwrap_or("Untitled").to_string();
        }
        Ok(config)
    }
}
