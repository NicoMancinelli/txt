use txt_core::fuzzy::FuzzyMatcher;

#[test]
fn test_fuzzy_exact_match() {
    let result = FuzzyMatcher::match_str("save", "File: Save");
    assert!(result.is_some());
    let match_data = result.unwrap();
    assert!(match_data.score > 50);
}

#[test]
fn test_fuzzy_acronym_sublime_matching() {
    // Sublime Text matching: 'cp' should strongly match 'Command Palette'
    let match1 = FuzzyMatcher::match_str("cp", "Tools: Command Palette");
    assert!(match1.is_some());

    let match2 = FuzzyMatcher::match_str("gst", "Git: Stash");
    assert!(match2.is_some());
}

#[test]
fn test_fuzzy_ranking() {
    let exact = FuzzyMatcher::match_str("open", "File: Open File").unwrap();
    let partial = FuzzyMatcher::match_str("open", "View: Reopen Closed Tab").unwrap();

    // "File: Open File" starts with 'Open' boundary and should outscore arbitrary substring
    assert!(exact.score >= partial.score);
}
