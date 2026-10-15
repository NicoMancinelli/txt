use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Git working tree and index status for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitFileStatus {
    Modified,
    Untracked,
    Added,
    Deleted,
    Renamed,
    Ignored,
    Clean,
}

impl GitFileStatus {
    /// Sublime Text / IDE single-character status badge.
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Untracked => "U",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Ignored => "I",
            Self::Clean => "",
        }
    }

    /// RGB color representation for gutter / file tree rendering (e.g. One Dark / Mariana inspired).
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Modified => (229, 192, 123),  // Warm amber / yellow
            Self::Untracked => (152, 195, 121), // Soft green
            Self::Added => (152, 195, 121),     // Green
            Self::Deleted => (224, 108, 117),   // Coral red
            Self::Renamed => (97, 175, 239),    // Soft blue
            Self::Ignored => (120, 120, 120),   // Subdued grey
            Self::Clean => (171, 178, 191),     // Neutral foreground
        }
    }

    /// Whether this file has uncommitted changes or is untracked.
    pub fn is_dirty(&self) -> bool {
        !matches!(self, Self::Clean | Self::Ignored)
    }

    /// Whether this file is completely clean.
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean)
    }
}

/// Scanner that detects git repositories and queries working tree file statuses.
#[derive(Debug, Default, Clone)]
pub struct GitStatusScanner;

impl GitStatusScanner {
    /// Creates a new GitStatusScanner.
    pub fn new() -> Self {
        Self
    }

    /// Searches upwards from `start_path` looking for a `.git` directory or file (for worktrees/submodules).
    pub fn find_repo_root(start_path: &Path) -> Option<PathBuf> {
        let mut current = if start_path.is_file() {
            start_path.parent()?
        } else {
            start_path
        };

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Some(current.canonicalize().unwrap_or_else(|_| current.to_path_buf()));
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
        None
    }

    /// Scans the repository for the given `repo_path` using `git status --porcelain=v1`.
    /// Returns a map of canonical file paths to their `GitFileStatus`.
    /// Returns an empty map if `repo_path` is not inside a git repository or git fails.
    pub fn scan_repository(repo_path: &Path) -> HashMap<PathBuf, GitFileStatus> {
        let repo_root = match Self::find_repo_root(repo_path) {
            Some(root) => root,
            None => return HashMap::new(),
        };

        let output = match Command::new("git")
            .arg("-c")
            .arg("core.quotePath=false")
            .arg("-C")
            .arg(&repo_root)
            .arg("status")
            .arg("--porcelain=v1")
            .arg("--untracked-files=all")
            .output()
        {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => return HashMap::new(),
        };

        Self::parse_porcelain_output(&output, &repo_root)
    }

    /// Parses the raw output of `git status --porcelain=v1` against a repository root.
    pub fn parse_porcelain_output(output: &str, repo_root: &Path) -> HashMap<PathBuf, GitFileStatus> {
        let canonical_root = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());
        let mut map = HashMap::new();

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }

            let index_code = line.as_bytes()[0] as char;
            let work_code = line.as_bytes()[1] as char;
            let path_part = line[3..].trim();

            let status = match (index_code, work_code) {
                ('?', '?') => GitFileStatus::Untracked,
                ('!', '!') => GitFileStatus::Ignored,
                ('R', _) | (_, 'R') => GitFileStatus::Renamed,
                ('D', _) | (_, 'D') => GitFileStatus::Deleted,
                ('A', _) | (_, 'A') => GitFileStatus::Added,
                ('M', _) | (_, 'M') => GitFileStatus::Modified,
                ('U', _) | (_, 'U') => GitFileStatus::Modified,
                ('C', _) | (_, 'C') => GitFileStatus::Added,
                _ => GitFileStatus::Modified,
            };

            // In renamed entries, format is "orig_path -> new_path"
            let target_rel_path = if let Some((_orig, new_path)) = path_part.split_once(" -> ") {
                new_path.trim()
            } else {
                path_part
            };

            let clean_rel = target_rel_path.trim_matches('"');
            let full_path = canonical_root.join(clean_rel);
            let canonical_path = full_path.canonicalize().unwrap_or(full_path);

            map.insert(canonical_path, status);
        }

        map
    }

    /// Returns the GitFileStatus of a specific file path within `repo_path`.
    pub fn get_file_status(repo_path: &Path, file_path: &Path) -> GitFileStatus {
        let canonical_target = file_path.canonicalize().unwrap_or_else(|_| file_path.to_path_buf());
        let map = Self::scan_repository(repo_path);
        map.get(&canonical_target).copied().unwrap_or(GitFileStatus::Clean)
    }
}
