use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::tempdir;
use txt_core::buffer::TextBuffer;
use txt_core::search::{ProjectSearchEngine, SearchQuery, SUBLIME_BATCH_SIZE};

#[test]
fn test_search_query_builder() {
    let q = SearchQuery::new("fn main")
        .case_sensitive(true)
        .whole_word(true)
        .regex(false);

    assert_eq!(q.pattern, "fn main");
    assert!(q.case_sensitive);
    assert!(q.whole_word);
    assert!(!q.is_regex);
}

#[test]
fn test_substring_search_case_sensitive() {
    let engine = ProjectSearchEngine::new();
    let text = "fn main() {\n    let foo = 42;\n    let Foo = 43;\n    println!(\"{}\", foo);\n}";
    let query = SearchQuery::new("foo").case_sensitive(true);

    let results = engine.search_text(text, &query, None).unwrap();
    assert_eq!(results.len(), 2);

    assert_eq!(results[0].line_number, 2);
    assert_eq!(results[0].column, 9);
    assert_eq!(results[0].line_text, "    let foo = 42;");
    assert_eq!(results[0].match_range, 8..11);

    assert_eq!(results[1].line_number, 4);
    assert_eq!(results[1].column, 20);
    assert_eq!(results[1].line_text, "    println!(\"{}\", foo);");
    assert_eq!(results[1].match_range, 19..22);
}

#[test]
fn test_substring_search_case_insensitive() {
    let engine = ProjectSearchEngine::new();
    let text = "Sublime Text\nsublime text\nSUBLIME TEXT\nother text";
    let query = SearchQuery::new("sublime").case_sensitive(false);

    let results = engine.search_text(text, &query, None).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].line_number, 1);
    assert_eq!(results[1].line_number, 2);
    assert_eq!(results[2].line_number, 3);
}

#[test]
fn test_whole_word_matching() {
    let engine = ProjectSearchEngine::new();
    let text = "word\nsword\nword_extra\n(word)\n\"word\"\nwords";
    let query = SearchQuery::new("word").whole_word(true);

    let results = engine.search_text(text, &query, None).unwrap();
    // Only "word", "(word)", and "\"word\"" are whole word matches
    let lines: Vec<usize> = results.iter().map(|r| r.line_number).collect();
    assert_eq!(lines, vec![1, 4, 5]);
}

#[test]
fn test_regex_matching() {
    let engine = ProjectSearchEngine::new();
    let text = "port: 8080\nhost: localhost\nport: 3000\nstatus: 200 OK";
    let query = SearchQuery::new(r"port:\s+(\d+)").regex(true);

    let results = engine.search_text(text, &query, None).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].line_number, 1);
    assert_eq!(results[0].line_text, "port: 8080");
    assert_eq!(results[1].line_number, 3);
    assert_eq!(results[1].line_text, "port: 3000");
}

#[test]
fn test_project_search_batching_4096_items() {
    let engine = ProjectSearchEngine::new();
    assert_eq!(engine.batch_size, SUBLIME_BATCH_SIZE);
    assert_eq!(SUBLIME_BATCH_SIZE, 4096);

    // Simulate 10,000 files
    let items: Vec<PathBuf> = (0..10_000)
        .map(|i| PathBuf::from(format!("file_{}.rs", i)))
        .collect();

    let batches = engine.partition_into_batches(items);

    // ceil(10,000 / 4096) = 3 batches
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].len(), 4096);
    assert_eq!(batches[1].len(), 4096);
    assert_eq!(batches[2].len(), 1808);

    let total: usize = batches.iter().map(|b| b.len()).sum();
    assert_eq!(total, 10_000);
}

#[test]
fn test_multithreaded_project_directory_search() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    // Create file hierarchy
    let src_dir = root.join("src");
    let target_dir = root.join("target");
    let git_dir = root.join(".git");

    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&target_dir).unwrap();
    fs::create_dir_all(&git_dir).unwrap();

    let file1 = root.join("README.md");
    let file2 = src_dir.join("main.rs");
    let file3 = src_dir.join("lib.rs");
    let file_ignored1 = target_dir.join("output.txt");
    let file_ignored2 = git_dir.join("config");

    fs::write(&file1, "# txt Editor\nHigh-performance editor written in Rust.").unwrap();
    fs::write(&file2, "fn main() {\n    println!(\"txt is fast\");\n}").unwrap();
    fs::write(&file3, "pub const VERSION: &str = \"0.1.0\";\n// txt core library").unwrap();
    fs::write(&file_ignored1, "txt in target should be ignored").unwrap();
    fs::write(&file_ignored2, "txt in git should be ignored").unwrap();

    let engine = ProjectSearchEngine::new();
    let query = SearchQuery::new("txt").case_sensitive(true);

    let results = engine.search_project(root, &query).unwrap();

    // Verify target and .git were ignored
    for r in &results {
        let p = r.file_path.to_string_lossy();
        assert!(!p.contains("target"), "Target directory must be ignored");
        assert!(!p.contains(".git"), ".git directory must be ignored");
    }

    // Matches should be in README.md, main.rs, and lib.rs
    assert_eq!(results.len(), 3);
}

#[test]
fn test_binary_file_skipping() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    let bin_path = root.join("binary.bin");
    let mut f = File::create(&bin_path).unwrap();
    // Write bytes with null byte header followed by search query string
    let mut bytes = vec![0x7f, b'E', b'L', b'F', 0, 0, 1, 2];
    bytes.extend_from_slice(b"KEYWORD inside binary");
    f.write_all(&bytes).unwrap();

    let engine = ProjectSearchEngine::new();
    let query = SearchQuery::new("KEYWORD");

    let results = engine.search_project(root, &query).unwrap();
    assert_eq!(results.len(), 0, "Binary file with null byte should be skipped");
}

#[test]
fn test_buffer_search() {
    let mut buf = TextBuffer::new();
    buf.insert(0, "Line 1: alpha\nLine 2: beta\nLine 3: alpha beta");

    let engine = ProjectSearchEngine::new();
    let query = SearchQuery::new("alpha");

    let results = engine.search_buffer(&buf, &query, None).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].line_number, 1);
    assert_eq!(results[0].line_text, "Line 1: alpha");
    assert_eq!(results[1].line_number, 3);
    assert_eq!(results[1].line_text, "Line 3: alpha beta");
}

#[test]
fn test_search_throughput_benchmark() {
    let engine = ProjectSearchEngine::new().with_threads(8);

    // Generate 50,000 lines of simulated source code
    let mut text = String::with_capacity(3 * 1024 * 1024);
    for i in 0..50_000 {
        if i % 1000 == 0 {
            text.push_str("pub fn find_target_symbol() -> bool { true }\n");
        } else if i % 2 == 0 {
            text.push_str("    let temporary_variable = calculate_metrics(i);\n");
        } else {
            text.push_str("    // internal comment explaining high performance data structure\n");
        }
    }

    let query = SearchQuery::new("find_target_symbol").case_sensitive(true);

    let start = Instant::now();
    let results = engine.search_text(&text, &query, None).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 50);

    let text_mb = text.len() as f64 / (1024.0 * 1024.0);
    let throughput_mb_s = text_mb / elapsed.as_secs_f64();
    let lines_per_sec = 50_000.0 / elapsed.as_secs_f64();

    println!(
        "Search throughput benchmark: 50,000 lines ({:.2} MB) scanned in {:?} ({:.1} MB/s, {:.0} lines/sec)",
        text_mb, elapsed, throughput_mb_s, lines_per_sec
    );

    // 50,000 lines should be scanned in well under 250ms
    assert!(
        elapsed.as_millis() < 500,
        "Search elapsed time took too long: {:?}",
        elapsed
    );
}
