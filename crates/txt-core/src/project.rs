use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Standard ignore list for projects (VCS and build artifacts).
pub const DEFAULT_IGNORED_NAMES: &[&str] = &[".git", "target", "node_modules", ".DS_Store"];

/// Represents a single file or directory node in the project tree hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub size_bytes: u64,
    pub is_hidden: bool,
}

impl TreeNode {
    /// Creates a new TreeNode.
    pub fn new(path: PathBuf, name: String, is_dir: bool, size_bytes: u64, is_hidden: bool) -> Self {
        Self {
            path,
            name,
            is_dir,
            children: Vec::new(),
            is_expanded: false,
            size_bytes,
            is_hidden,
        }
    }

    /// Recursively counts total nodes in this subtree (including self).
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    /// Recursively searches for a node by absolute or relative path.
    pub fn find_node(&self, target_path: &Path) -> Option<&TreeNode> {
        if self.path == target_path {
            return Some(self);
        }
        if self.is_dir {
            for child in &self.children {
                if let Some(found) = child.find_node(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Recursively searches for a mutable node reference by path.
    pub fn find_node_mut(&mut self, target_path: &Path) -> Option<&mut TreeNode> {
        if self.path == target_path {
            return Some(self);
        }
        if self.is_dir {
            for child in &mut self.children {
                if let Some(found) = child.find_node_mut(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Sets the expanded state for the matching node path.
    pub fn set_expanded(&mut self, target_path: &Path, expanded: bool) -> bool {
        if self.path == target_path {
            self.is_expanded = expanded;
            return true;
        }
        if self.is_dir {
            for child in &mut self.children {
                if child.set_expanded(target_path, expanded) {
                    return true;
                }
            }
        }
        false
    }

    /// Sorts children placing directories first, then alphabetical by name (case-insensitive).
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a
                    .name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then_with(|| a.name.cmp(&b.name)),
            }
        });
        for child in &mut self.children {
            if child.is_dir {
                child.sort_children();
            }
        }
    }

    /// Flattens the visible tree based on current `is_expanded` flags in pre-order traversal.
    pub fn flatten_visible(&self) -> Vec<&TreeNode> {
        let mut list = Vec::new();
        self.collect_visible(&mut list);
        list
    }

    fn collect_visible<'a>(&'a self, list: &mut Vec<&'a TreeNode>) {
        list.push(self);
        if self.is_dir && self.is_expanded {
            for child in &self.children {
                child.collect_visible(list);
            }
        }
    }
}

/// A root folder in the project workspace with expansion and scanning state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFolder {
    pub root_path: PathBuf,
    pub name: String,
    pub root_node: TreeNode,
    pub expanded_paths: HashSet<PathBuf>,
}

impl ProjectFolder {
    /// Creates a new ProjectFolder and scans its filesystem tree.
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.to_str().unwrap_or("project"))
            .to_string();

        let mut expanded_paths = HashSet::new();
        expanded_paths.insert(path.clone());

        let is_hidden = name.starts_with('.');
        let root_node = TreeNode {
            path: path.clone(),
            name: name.clone(),
            is_dir: true,
            children: Vec::new(),
            is_expanded: true,
            size_bytes: 0,
            is_hidden,
        };

        let mut folder = Self {
            root_path: path,
            name,
            root_node,
            expanded_paths,
        };
        folder.refresh(false, None);
        folder
    }

    /// Re-scans the filesystem tree under `root_path`.
    ///
    /// Preserves expansion state from `expanded_paths`.
    /// If `show_hidden` is false, ignores standard ignore patterns and dotfiles.
    /// If `filter` is provided, prunes nodes whose names or subtree do not match.
    pub fn refresh(&mut self, show_hidden: bool, filter: Option<&str>) {
        let is_hidden = self.name.starts_with('.');
        let is_expanded = self.expanded_paths.contains(&self.root_path);

        let mut root = TreeNode {
            path: self.root_path.clone(),
            name: self.name.clone(),
            is_dir: true,
            children: Vec::new(),
            is_expanded,
            size_bytes: 0,
            is_hidden,
        };

        if self.root_path.is_dir() {
            root.children = scan_directory(&self.root_path, show_hidden, &self.expanded_paths);
            root.sort_children();
        }

        if let Some(pattern) = filter {
            let pattern = pattern.trim();
            if !pattern.is_empty() {
                filter_node_children(&mut root, pattern);
            }
        }

        self.root_node = root;
    }

    /// Toggles the expansion state of the directory at `path`.
    pub fn toggle_expand(&mut self, path: &Path) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
            self.root_node.set_expanded(path, false);
        } else {
            self.expanded_paths.insert(path.to_path_buf());
            self.root_node.set_expanded(path, true);
        }
    }

    /// Recursively expands all directory nodes in the folder tree.
    pub fn expand_all(&mut self) {
        fn expand_recursive(node: &mut TreeNode, expanded: &mut HashSet<PathBuf>) {
            if node.is_dir {
                node.is_expanded = true;
                expanded.insert(node.path.clone());
                for child in &mut node.children {
                    expand_recursive(child, expanded);
                }
            }
        }
        expand_recursive(&mut self.root_node, &mut self.expanded_paths);
    }

    /// Recursively collapses all directory nodes in the folder tree.
    pub fn collapse_all(&mut self) {
        fn collapse_recursive(node: &mut TreeNode) {
            if node.is_dir {
                node.is_expanded = false;
                for child in &mut node.children {
                    collapse_recursive(child);
                }
            }
        }
        self.expanded_paths.clear();
        collapse_recursive(&mut self.root_node);
    }

    /// Reveals a file or directory path in the sidebar tree.
    ///
    /// Expands all ancestor directories leading up to `path`.
    /// Returns `true` if the path is contained in this folder, `false` otherwise.
    pub fn reveal_path(&mut self, path: &Path) -> bool {
        if !path.starts_with(&self.root_path) {
            return false;
        }

        // Verify the path exists either on disk or in the current tree
        if !path.exists() && self.root_node.find_node(path).is_none() {
            return false;
        }

        // Expand root folder
        self.root_node.is_expanded = true;
        self.expanded_paths.insert(self.root_path.clone());

        // Traverse ancestors from path up to root_path and expand each directory
        let mut current = path.parent();
        while let Some(parent) = current {
            if parent == self.root_path {
                break;
            }
            if parent.starts_with(&self.root_path) {
                self.expanded_paths.insert(parent.to_path_buf());
                self.root_node.set_expanded(parent, true);
                current = parent.parent();
            } else {
                break;
            }
        }

        true
    }
}

/// Project manager managing multi-root workspace folders and global project tree state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManager {
    pub folders: Vec<ProjectFolder>,
    pub show_hidden: bool,
}

impl ProjectManager {
    /// Creates a new empty ProjectManager.
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            show_hidden: false,
        }
    }

    /// Adds a folder to the project manager. If already present, refreshes it.
    pub fn add_folder(&mut self, path: PathBuf) {
        if let Some(existing) = self.folders.iter_mut().find(|f| f.root_path == path) {
            existing.refresh(self.show_hidden, None);
        } else {
            let mut folder = ProjectFolder::new(path);
            folder.refresh(self.show_hidden, None);
            self.folders.push(folder);
        }
    }

    /// Removes a folder from the project manager by path.
    pub fn remove_folder(&mut self, path: &Path) {
        self.folders.retain(|f| f.root_path != path);
    }

    /// Refreshes all project folders with optional filter query.
    pub fn refresh_all(&mut self, filter: Option<&str>) {
        for folder in &mut self.folders {
            folder.refresh(self.show_hidden, filter);
        }
    }

    /// Reveals a path across all managed folders.
    ///
    /// Returns `true` if the path belongs to one of the folders and was revealed.
    pub fn reveal_path(&mut self, path: &Path) -> bool {
        for folder in &mut self.folders {
            if folder.reveal_path(path) {
                return true;
            }
        }
        false
    }

    /// Toggles showing hidden files and refreshes all folders.
    pub fn toggle_show_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh_all(None);
    }
}

/// Atomic file system operations for files and directories.
pub struct FileOps;

impl FileOps {
    /// Creates an empty file at `path`, creating parent directories if necessary.
    pub fn create_file(path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::File::create(path)?;
        Ok(())
    }

    /// Creates a directory and any missing parent directories.
    pub fn create_dir(path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    /// Renames a file or directory from `from` to `to`, creating destination parent dirs if needed.
    pub fn rename(from: &Path, to: &Path) -> io::Result<()> {
        if !from.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path '{}' does not exist", from.display()),
            ));
        }
        if let Some(parent) = to.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::rename(from, to)
    }

    /// Duplicates a file or directory.
    ///
    /// Generates non-colliding copy names:
    /// e.g. `foo.rs` -> `foo_copy.rs` or `foo (1).rs` if destination exists.
    pub fn duplicate(src: &Path) -> io::Result<PathBuf> {
        if !src.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path '{}' does not exist", src.display()),
            ));
        }

        let parent = src.parent().unwrap_or_else(|| Path::new(""));
        let file_name = src
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;

        let is_dir = src.is_dir();
        let (stem, ext) = if is_dir {
            (file_name, None)
        } else {
            let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
            let ext = src.extension().and_then(|e| e.to_str());
            (stem, ext)
        };

        // Attempt 1: <stem>_copy.<ext>
        let candidate_1 = match ext {
            Some(e) => format!("{}_copy.{}", stem, e),
            None => format!("{}_copy", stem),
        };
        let dest_1 = parent.join(&candidate_1);

        let dest = if !dest_1.exists() {
            dest_1
        } else {
            // Attempt 2+: <stem> (1).<ext>, <stem> (2).<ext>, ...
            let mut index = 1;
            loop {
                let candidate_n = match ext {
                    Some(e) => format!("{} ({}).{}", stem, index, e),
                    None => format!("{} ({})", stem, index),
                };
                let dest_n = parent.join(&candidate_n);
                if !dest_n.exists() {
                    break dest_n;
                }
                index += 1;
            }
        };

        if is_dir {
            copy_dir_recursive(src, &dest)?;
        } else {
            fs::copy(src, &dest)?;
        }

        Ok(dest)
    }

    /// Deletes a file or directory (recursively if directory).
    pub fn delete(path: &Path) -> io::Result<()> {
        if !path.exists() && !path.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path '{}' does not exist", path.display()),
            ));
        }
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn is_ignored(name: &str, show_hidden: bool) -> bool {
    if show_hidden {
        return false;
    }
    if DEFAULT_IGNORED_NAMES.contains(&name) {
        return true;
    }
    name.starts_with('.')
}

fn scan_directory(dir_path: &Path, show_hidden: bool, expanded_paths: &HashSet<PathBuf>) -> Vec<TreeNode> {
    let mut entries = Vec::new();

    let read_res = match fs::read_dir(dir_path) {
        Ok(iter) => iter,
        Err(_) => return entries,
    };

    for entry in read_res.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if is_ignored(&name, show_hidden) {
            continue;
        }

        let path = entry.path();
        let (is_dir, size_bytes) = match entry.metadata() {
            Ok(meta) => (meta.is_dir(), if meta.is_dir() { 0 } else { meta.len() }),
            Err(_) => (false, 0),
        };

        let is_hidden = name.starts_with('.');
        let is_expanded = is_dir && expanded_paths.contains(&path);

        let children = if is_dir {
            scan_directory(&path, show_hidden, expanded_paths)
        } else {
            Vec::new()
        };

        entries.push(TreeNode {
            path,
            name,
            is_dir,
            children,
            is_expanded,
            size_bytes,
            is_hidden,
        });
    }

    entries
}

fn filter_node_children(node: &mut TreeNode, filter: &str) {
    let mut filtered = Vec::new();
    for mut child in std::mem::take(&mut node.children) {
        if filter_tree(&mut child, filter) {
            filtered.push(child);
        }
    }
    node.children = filtered;
}

fn filter_tree(node: &mut TreeNode, filter: &str) -> bool {
    let filter_lower = filter.to_lowercase();
    let name_matches = node.name.to_lowercase().contains(&filter_lower);

    if node.is_dir {
        let mut matching_children = Vec::new();
        for mut child in std::mem::take(&mut node.children) {
            if filter_tree(&mut child, filter) {
                matching_children.push(child);
            }
        }
        node.children = matching_children;
        name_matches || !node.children.is_empty()
    } else {
        name_matches
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
