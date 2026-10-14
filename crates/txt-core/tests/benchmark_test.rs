use std::time::Instant;
use txt_core::buffer::TextBuffer;
use txt_core::fuzzy::FuzzyMatcher;

#[test]
fn test_buffer_high_frequency_inserts_throughput() {
    let mut buf = TextBuffer::new();
    let start = Instant::now();

    // 10,000 rapid keystroke insertions
    for i in 0..10_000 {
        buf.insert(buf.len_chars(), "a");
        if i % 100 == 0 {
            buf.insert(buf.len_chars(), "\n");
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(buf.len_chars(), 10_100);
    // 10,000 keystrokes should take under 50ms in release/debug on modern Apple Silicon
    println!("10,000 rapid buffer edits executed in: {:?}", elapsed);
    assert!(elapsed.as_millis() < 500);
}

#[test]
fn test_fuzzy_matcher_throughput() {
    let queries = ["save", "cp", "op", "theme", "sync", "new"];
    let targets = [
        "File: Save",
        "File: Open File",
        "File: New File",
        "Tools: Command Palette",
        "Preferences: Theme: Mariana",
        "Preferences: Theme: Monokai",
        "Sync: Sync Document Now",
        "Sync: Toggle Cloud Offline/Online",
    ];

    let start = Instant::now();
    for _ in 0..5_000 {
        for q in queries {
            for t in targets {
                let _ = FuzzyMatcher::match_str(q, t);
            }
        }
    }
    let elapsed = start.elapsed();
    println!("240,000 fuzzy evaluations executed in: {:?}", elapsed);
    assert!(elapsed.as_millis() < 1000);
}
