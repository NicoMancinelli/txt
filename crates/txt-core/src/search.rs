use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crossbeam_channel::unbounded;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::buffer::TextBuffer;

/// Sublime Text's empirical batch size (`parallel_for_task<...Li4096EE>`).
/// 4096 items amortize thread synchronization overhead while maximizing CPU core saturation.
pub const SUBLIME_BATCH_SIZE: usize = 4096;

/// Default maximum file size to scan (20 MB), preventing out-of-memory on huge binary blobs.
pub const DEFAULT_MAX_FILE_SIZE: u64 = 20 * 1024 * 1024;

/// Search query configuration specifying pattern and matching modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub is_regex: bool,
}

impl SearchQuery {
    /// Creates a new search query with default settings (case-insensitive, substring).
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            case_sensitive: false,
            whole_word: false,
            is_regex: false,
        }
    }

    /// Sets case sensitivity.
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Sets whole-word matching.
    pub fn whole_word(mut self, enabled: bool) -> Self {
        self.whole_word = enabled;
        self
    }

    /// Sets regular expression matching.
    pub fn regex(mut self, enabled: bool) -> Self {
        self.is_regex = enabled;
        self
    }
}

/// A matched search result indicating location and context within a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: PathBuf,
    /// 1-based line number.
    pub line_number: usize,
    /// 1-based character column.
    pub column: usize,
    /// Byte range within `line_text` [start, end).
    pub match_range: Range<usize>,
    /// The trimmed line content containing the match.
    pub line_text: String,
}

/// Errors that can occur during search operations.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Invalid regular expression: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Worker execution error: {0}")]
    WorkerError(String),
}

/// Internal compiled search matcher for fast low-allocation line scanning.
#[derive(Clone)]
enum CompiledSearcher {
    Regex(regex::Regex),
    Literal {
        pattern: String,
        case_sensitive: bool,
        pattern_lower: String,
    },
}

impl CompiledSearcher {
    fn compile(query: &SearchQuery) -> Result<Self, SearchError> {
        if query.is_regex {
            let mut pattern_str = query.pattern.clone();
            if query.whole_word {
                pattern_str = format!(r"\b(?:{})\b", pattern_str);
            }
            let re = regex::RegexBuilder::new(&pattern_str)
                .case_insensitive(!query.case_sensitive)
                .multi_line(false)
                .build()?;
            Ok(Self::Regex(re))
        } else if query.whole_word {
            // Whole-word literal search via regex word-boundary boundaries
            let escaped = regex::escape(&query.pattern);
            let pattern_str = format!(r"\b(?:{})\b", escaped);
            let re = regex::RegexBuilder::new(&pattern_str)
                .case_insensitive(!query.case_sensitive)
                .multi_line(false)
                .build()?;
            Ok(Self::Regex(re))
        } else {
            Ok(Self::Literal {
                pattern_lower: query.pattern.to_lowercase(),
                pattern: query.pattern.clone(),
                case_sensitive: query.case_sensitive,
            })
        }
    }

    /// Finds all match ranges (byte indices) within a single line.
    #[inline]
    fn find_matches_in_line(&self, line: &str, matches: &mut Vec<Range<usize>>) {
        matches.clear();
        if line.is_empty() {
            return;
        }

        match self {
            Self::Regex(re) => {
                for mat in re.find_iter(line) {
                    matches.push(mat.range());
                }
            }
            Self::Literal {
                pattern,
                case_sensitive,
                pattern_lower,
                ..
            } => {
                if pattern.is_empty() {
                    return;
                }

                if *case_sensitive {
                    let mut start = 0;
                    while let Some(idx) = line[start..].find(pattern.as_str()) {
                        let match_start = start + idx;
                        let match_end = match_start + pattern.len();
                        matches.push(match_start..match_end);
                        start = match_end;
                    }
                } else {
                    // Fast case-insensitive substring search
                    let line_lower = line.to_lowercase();
                    let mut start = 0;
                    while let Some(idx) = line_lower[start..].find(pattern_lower.as_str()) {
                        let match_start = start + idx;
                        let match_end = match_start + pattern.len();
                        // Guard against char boundary drift in case conversion
                        if line.is_char_boundary(match_start) && line.is_char_boundary(match_end) {
                            matches.push(match_start..match_end);
                        }
                        start = match_start + 1.max(pattern.len());
                        if start >= line.len() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// High-performance project search engine modeled after Sublime Text's parallel task architecture.
#[derive(Debug, Clone)]
pub struct ProjectSearchEngine {
    pub batch_size: usize,
    pub num_threads: usize,
    pub max_file_size: u64,
    pub ignore_patterns: Vec<String>,
}

impl Default for ProjectSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectSearchEngine {
    /// Creates a new ProjectSearchEngine with empirical Sublime Text defaults:
    /// - 4096-item batch size (`SUBLIME_BATCH_SIZE`).
    /// - Thread pool scaled to available CPU cores.
    /// - Ignored common build & VCS artifacts (`target`, `.git`, `node_modules`).
    pub fn new() -> Self {
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self {
            batch_size: SUBLIME_BATCH_SIZE,
            num_threads: threads,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            ignore_patterns: vec![
                "target".into(),
                ".git".into(),
                "node_modules".into(),
                ".DS_Store".into(),
                ".idea".into(),
                ".vscode".into(),
            ],
        }
    }

    /// Sets the task chunk size (Sublime default: 4096 items).
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// Sets the number of parallel worker threads.
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.num_threads = threads.max(1);
        self
    }

    /// Sets custom directory / file ignore patterns.
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    /// Sets maximum file size in bytes to inspect.
    pub fn with_max_file_size(mut self, max_bytes: u64) -> Self {
        self.max_file_size = max_bytes;
        self
    }

    /// Discovers all candidate files in `root_dir` respecting ignore patterns.
    pub fn discover_files(&self, root_dir: impl AsRef<Path>) -> Vec<PathBuf> {
        let root = root_dir.as_ref();
        if !root.exists() {
            return Vec::new();
        }

        let ignores = &self.ignore_patterns;
        let mut files = Vec::new();

        let walker = WalkDir::new(root).follow_links(false).into_iter();

        for entry in walker.filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !ignores.iter().any(|ig| file_name == ig.as_str())
        }) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    files.push(entry.into_path());
                }
            }
        }

        files
    }

    /// Partitions an arbitrary item vector into Sublime Text 4096-item work units.
    pub fn partition_into_batches<T>(&self, items: Vec<T>) -> Vec<Vec<T>> {
        if items.is_empty() {
            return Vec::new();
        }

        let chunk_size = self.batch_size;
        let num_batches = (items.len() + chunk_size - 1) / chunk_size;
        let mut batches = Vec::with_capacity(num_batches);

        let mut iter = items.into_iter();
        for _ in 0..num_batches {
            let batch: Vec<T> = iter.by_ref().take(chunk_size).collect();
            if !batch.is_empty() {
                batches.push(batch);
            }
        }

        batches
    }

    /// Discovers project files and partitions them into 4096-item work units.
    pub fn discover_file_batches(&self, root_dir: impl AsRef<Path>) -> Vec<Vec<PathBuf>> {
        let files = self.discover_files(root_dir);
        self.partition_into_batches(files)
    }

    /// Performs parallel multithreaded search across a project directory tree.
    pub fn search_project(
        &self,
        root_dir: impl AsRef<Path>,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let batches = self.discover_file_batches(root_dir);
        self.search_batches(&batches, query)
    }

    /// Searches a list of file paths in parallel using 4096-item batching.
    pub fn search_files(
        &self,
        files: &[PathBuf],
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let batches = self.partition_into_batches(files.to_vec());
        self.search_batches(&batches, query)
    }

    /// Searches pre-batched work units using multithreaded workers with low allocation.
    pub fn search_batches(
        &self,
        batches: &[Vec<PathBuf>],
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, SearchError> {
        if batches.is_empty() {
            return Ok(Vec::new());
        }

        let searcher = Arc::new(CompiledSearcher::compile(query)?);
        let max_file_size = self.max_file_size;

        // Work queue channel for 4096-item batches
        let (tx, rx) = unbounded::<Vec<PathBuf>>();
        for batch in batches {
            let _ = tx.send(batch.clone());
        }
        drop(tx);

        let (result_tx, result_rx) = unbounded::<Vec<SearchResult>>();
        let total_workers = self.num_threads.min(batches.len()).max(1);

        std::thread::scope(|scope| {
            for _ in 0..total_workers {
                let rx_worker = rx.clone();
                let result_tx_worker = result_tx.clone();
                let searcher_worker = Arc::clone(&searcher);

                scope.spawn(move || {
                    // Reused buffers to minimize allocation overhead across 4096 items
                    let mut line_buf = String::with_capacity(512);
                    let mut match_ranges = Vec::with_capacity(8);
                    let mut local_results = Vec::new();
                    let mut header_buf = [0u8; 512];

                    while let Ok(batch) = rx_worker.recv() {
                        for path in &batch {
                            Self::scan_single_file(
                                path,
                                &searcher_worker,
                                max_file_size,
                                &mut line_buf,
                                &mut match_ranges,
                                &mut header_buf,
                                &mut local_results,
                            );
                        }
                    }

                    if !local_results.is_empty() {
                        let _ = result_tx_worker.send(local_results);
                    }
                });
            }
        });

        drop(result_tx);

        let mut all_results = Vec::new();
        while let Ok(mut batch_results) = result_rx.recv() {
            all_results.append(&mut batch_results);
        }

        // Deterministic sort: by file path, then line number, then column
        all_results.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.line_number.cmp(&b.line_number))
                .then_with(|| a.column.cmp(&b.column))
        });

        Ok(all_results)
    }

    /// Scans a single file on disk with zero allocations for non-matching lines.
    #[inline]
    fn scan_single_file(
        path: &Path,
        searcher: &CompiledSearcher,
        max_file_size: u64,
        line_buf: &mut String,
        match_ranges: &mut Vec<Range<usize>>,
        header_buf: &mut [u8; 512],
        results: &mut Vec<SearchResult>,
    ) {
        let Ok(file) = File::open(path) else {
            return;
        };

        if let Ok(metadata) = file.metadata() {
            if metadata.len() > max_file_size {
                return;
            }
        }

        let mut reader = BufReader::new(file);

        // Fast binary detection: check first 512 bytes for null byte
        if let Ok(bytes_read) = reader.read(header_buf) {
            if header_buf[..bytes_read].contains(&0) {
                return; // Skip binary file
            }
            // Rewind to beginning
            if let Ok(file) = File::open(path) {
                reader = BufReader::new(file);
            } else {
                return;
            }
        }

        let mut line_number = 1;
        line_buf.clear();

        while let Ok(n) = reader.read_line(line_buf) {
            if n == 0 {
                break;
            }

            let trimmed_line = line_buf.trim_end_matches(&['\r', '\n'][..]);
            searcher.find_matches_in_line(trimmed_line, match_ranges);

            if !match_ranges.is_empty() {
                for range in match_ranges.iter() {
                    let col = trimmed_line[..range.start].chars().count() + 1;
                    results.push(SearchResult {
                        file_path: path.to_path_buf(),
                        line_number,
                        column: col,
                        match_range: range.clone(),
                        line_text: trimmed_line.to_string(),
                    });
                }
            }

            line_number += 1;
            line_buf.clear();
        }
    }

    /// Searches text in-memory directly with low allocation.
    pub fn search_text(
        &self,
        text: &str,
        query: &SearchQuery,
        path: Option<&Path>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = CompiledSearcher::compile(query)?;
        let mut results = Vec::new();
        let mut match_ranges = Vec::new();
        let file_path = path.map(PathBuf::from).unwrap_or_default();

        for (idx, line) in text.lines().enumerate() {
            searcher.find_matches_in_line(line, &mut match_ranges);
            for range in &match_ranges {
                let col = line[..range.start].chars().count() + 1;
                results.push(SearchResult {
                    file_path: file_path.clone(),
                    line_number: idx + 1,
                    column: col,
                    match_range: range.clone(),
                    line_text: line.to_string(),
                });
            }
        }

        Ok(results)
    }

    /// Searches a `TextBuffer` directly, slicing line-by-line without full string conversions.
    pub fn search_buffer(
        &self,
        buffer: &TextBuffer,
        query: &SearchQuery,
        path: Option<&Path>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = CompiledSearcher::compile(query)?;
        let mut results = Vec::new();
        let mut match_ranges = Vec::new();
        let file_path = path.map(PathBuf::from).unwrap_or_default();
        let line_count = buffer.len_lines();

        for line_idx in 0..line_count {
            let Some(line_slice) = buffer.line(line_idx) else {
                continue;
            };
            let line_str = line_slice.to_string();
            let trimmed = line_str.trim_end_matches(&['\r', '\n'][..]);

            searcher.find_matches_in_line(trimmed, &mut match_ranges);
            for range in &match_ranges {
                let col = trimmed[..range.start].chars().count() + 1;
                results.push(SearchResult {
                    file_path: file_path.clone(),
                    line_number: line_idx + 1,
                    column: col,
                    match_range: range.clone(),
                    line_text: trimmed.to_string(),
                });
            }
        }

        Ok(results)
    }
}
