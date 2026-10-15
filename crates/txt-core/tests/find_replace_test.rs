use txt_core::buffer::TextBuffer;
use txt_core::find_replace::{FindOptions, FindReplaceEngine};

#[test]
fn test_plain_case_sensitive_matching() {
    let text = "fn main() {\n    let foo = 42;\n    let Foo = 43;\n    println!(\"{}\", foo);\n}";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().case_sensitive(true);
    let matches = FindReplaceEngine::find_all(&buffer, "foo", &options);

    assert_eq!(matches.len(), 2);

    // First match on line 1: "    let foo = 42;"
    assert_eq!(matches[0].text, "foo");
    assert_eq!(matches[0].line, 1);
    assert_eq!(matches[0].column, 8);
    assert_eq!(buffer.slice_text(matches[0].range.clone()), "foo");

    // Second match on line 3: "    println!("{}", foo);"
    assert_eq!(matches[1].text, "foo");
    assert_eq!(matches[1].line, 3);
    assert_eq!(matches[1].column, 19);
    assert_eq!(buffer.slice_text(matches[1].range.clone()), "foo");
}

#[test]
fn test_plain_case_insensitive_matching() {
    let text = "Sublime Text\nsublime text\nSUBLIME TEXT\nother text";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().case_sensitive(false);
    let matches = FindReplaceEngine::find_all(&buffer, "sublime", &options);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].line, 0);
    assert_eq!(matches[0].text, "Sublime");
    assert_eq!(matches[1].line, 1);
    assert_eq!(matches[1].text, "sublime");
    assert_eq!(matches[2].line, 2);
    assert_eq!(matches[2].text, "SUBLIME");
}

#[test]
fn test_whole_word_matching() {
    let text = "word\nsword\nword_extra\n(word)\n\"word\"\nwords";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().whole_word(true);
    let matches = FindReplaceEngine::find_all(&buffer, "word", &options);

    // Only "word", "(word)", and "\"word\"" are whole word matches
    let lines: Vec<usize> = matches.iter().map(|m| m.line).collect();
    assert_eq!(lines, vec![0, 3, 4]);

    for m in &matches {
        assert_eq!(m.text, "word");
    }
}

#[test]
fn test_whole_word_case_insensitive() {
    let text = "Apple pineapple apple APPLE (apple)";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().whole_word(true).case_sensitive(false);
    let matches = FindReplaceEngine::find_all(&buffer, "apple", &options);

    assert_eq!(matches.len(), 4);
    let texts: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, vec!["Apple", "apple", "APPLE", "apple"]);
}

#[test]
fn test_regex_pattern_matching() {
    let text = "port: 8080\nhost: localhost\nport: 3000\nstatus: 200 OK";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().regex(true);
    let matches = FindReplaceEngine::find_all(&buffer, r"port:\s+(\d+)", &options);

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].line, 0);
    assert_eq!(matches[0].text, "port: 8080");
    assert_eq!(matches[1].line, 2);
    assert_eq!(matches[1].text, "port: 3000");
}

#[test]
fn test_regex_multiline_anchors() {
    let text = "first line\nsecond line\nthird line";
    let buffer = TextBuffer::from_str(text);

    let options = FindOptions::new().regex(true);
    let matches = FindReplaceEngine::find_all(&buffer, r"^(\w+)", &options);

    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].text, "first");
    assert_eq!(matches[1].text, "second");
    assert_eq!(matches[2].text, "third");
}

#[test]
fn test_invalid_regex_graceful_handling() {
    let buffer = TextBuffer::from_str("some content");
    let options = FindOptions::new().regex(true);

    // Invalid regex unclosed parenthesis
    let matches = FindReplaceEngine::find_all(&buffer, "(unclosed", &options);
    assert!(matches.is_empty());

    let next = FindReplaceEngine::find_next(&buffer, "(unclosed", 0, &options);
    assert!(next.is_none());

    let prev = FindReplaceEngine::find_prev(&buffer, "(unclosed", 0, &options);
    assert!(prev.is_none());
}

#[test]
fn test_find_next_wrap_around() {
    let text = "apple banana apple orange apple";
    let buffer = TextBuffer::from_str(text);
    let options = FindOptions::new().case_sensitive(true);

    // Match 0: 0..5
    // Match 1: 13..18
    // Match 2: 26..31

    // From 0 finds the first match at 0..5
    let m1 = FindReplaceEngine::find_next(&buffer, "apple", 0, &options).unwrap();
    assert_eq!(m1.range, 0..5);

    // From 5 (after first match) finds the second match at 13..18
    let m2 = FindReplaceEngine::find_next(&buffer, "apple", 5, &options).unwrap();
    assert_eq!(m2.range, 13..18);

    // From 18 finds the third match at 26..31
    let m3 = FindReplaceEngine::find_next(&buffer, "apple", 18, &options).unwrap();
    assert_eq!(m3.range, 26..31);

    // From 31 (at/past EOF of matches) wraps around to the first match at 0..5
    let m_wrapped = FindReplaceEngine::find_next(&buffer, "apple", 31, &options).unwrap();
    assert_eq!(m_wrapped.range, 0..5);

    // From mid-buffer beyond last match wraps around to 0
    let m_wrapped2 = FindReplaceEngine::find_next(&buffer, "apple", 100, &options).unwrap();
    assert_eq!(m_wrapped2.range, 0..5);
}

#[test]
fn test_find_prev_wrap_around() {
    let text = "apple banana apple orange apple";
    let buffer = TextBuffer::from_str(text);
    let options = FindOptions::new().case_sensitive(true);

    // Match 0: 0..5
    // Match 1: 13..18
    // Match 2: 26..31

    // From end of buffer (31) finds the previous match at 26..31
    let m3 = FindReplaceEngine::find_prev(&buffer, "apple", 31, &options).unwrap();
    assert_eq!(m3.range, 26..31);

    // From 26 (start of third match) finds the previous match at 13..18
    let m2 = FindReplaceEngine::find_prev(&buffer, "apple", 26, &options).unwrap();
    assert_eq!(m2.range, 13..18);

    // From 13 (start of second match) finds the previous match at 0..5
    let m1 = FindReplaceEngine::find_prev(&buffer, "apple", 13, &options).unwrap();
    assert_eq!(m1.range, 0..5);

    // From 0 (at beginning of buffer) wraps around to the last match at 26..31
    let m_wrapped = FindReplaceEngine::find_prev(&buffer, "apple", 0, &options).unwrap();
    assert_eq!(m_wrapped.range, 26..31);
}

#[test]
fn test_single_match_wrap_around() {
    let text = "hello world";
    let buffer = TextBuffer::from_str(text);
    let options = FindOptions::new();

    // Next from 0 finds "world" at 6..11
    let next1 = FindReplaceEngine::find_next(&buffer, "world", 0, &options).unwrap();
    assert_eq!(next1.range, 6..11);

    // Next from 11 wraps around to find "world" at 6..11
    let next2 = FindReplaceEngine::find_next(&buffer, "world", 11, &options).unwrap();
    assert_eq!(next2.range, 6..11);

    // Prev from 11 finds "world" at 6..11
    let prev1 = FindReplaceEngine::find_prev(&buffer, "world", 11, &options).unwrap();
    assert_eq!(prev1.range, 6..11);

    // Prev from 0 wraps around to find "world" at 6..11
    let prev2 = FindReplaceEngine::find_prev(&buffer, "world", 0, &options).unwrap();
    assert_eq!(prev2.range, 6..11);
}

#[test]
fn test_replace_next() {
    let mut buffer = TextBuffer::from_str("foo bar foo baz");
    let options = FindOptions::new().case_sensitive(true);

    // Replace first "foo" starting from 0
    let rep_range = FindReplaceEngine::replace_next(&mut buffer, "foo", "qux", 0, &options).unwrap();
    assert_eq!(rep_range, 0..3);
    assert_eq!(buffer.text(), "qux bar foo baz");

    // Replace next "foo" starting from 3
    let rep_range2 = FindReplaceEngine::replace_next(&mut buffer, "foo", "long_replacement", 3, &options).unwrap();
    assert_eq!(rep_range2, 8..24);
    assert_eq!(buffer.text(), "qux bar long_replacement baz");

    // No more "foo", returns None
    let rep_none = FindReplaceEngine::replace_next(&mut buffer, "foo", "nope", 0, &options);
    assert!(rep_none.is_none());
}

#[test]
fn test_replace_all_with_count_validation() {
    let mut buffer = TextBuffer::from_str("alpha beta alpha gamma alpha");
    let options = FindOptions::new().case_sensitive(true);

    let count = FindReplaceEngine::replace_all(&mut buffer, "alpha", "OMEGA", &options);
    assert_eq!(count, 3);
    assert_eq!(buffer.text(), "OMEGA beta OMEGA gamma OMEGA");
}

#[test]
fn test_replace_all_different_lengths() {
    // Shorter replacement
    let mut buffer = TextBuffer::from_str("longword 1 longword 2 longword 3");
    let options = FindOptions::new().case_sensitive(true);

    let count = FindReplaceEngine::replace_all(&mut buffer, "longword", "w", &options);
    assert_eq!(count, 3);
    assert_eq!(buffer.text(), "w 1 w 2 w 3");

    // Empty replacement (deletion)
    let mut buffer2 = TextBuffer::from_str("remove me, remove me!");
    let count2 = FindReplaceEngine::replace_all(&mut buffer2, "remove me", "", &options);
    assert_eq!(count2, 2);
    assert_eq!(buffer2.text(), ", !");
}

#[test]
fn test_replace_with_regex_captures() {
    let mut buffer = TextBuffer::from_str("item: 100\nitem: 200\nitem: 300");
    let options = FindOptions::new().regex(true);

    let count = FindReplaceEngine::replace_all(&mut buffer, r"item:\s*(\d+)", "val=$1", &options);
    assert_eq!(count, 3);
    assert_eq!(buffer.text(), "val=100\nval=200\nval=300");
}

#[test]
fn test_unicode_utf8_char_indexing() {
    let text = "🦀 Crab 🦀 Crab 🦀";
    let mut buffer = TextBuffer::from_str(text);
    let options = FindOptions::new();

    let matches = FindReplaceEngine::find_all(&buffer, "Crab", &options);
    assert_eq!(matches.len(), 2);

    // Each crab emoji 🦀 is 1 char in Ropey, so:
    // '🦀' (0), ' ' (1), 'C' (2), 'r' (3), 'a' (4), 'b' (5), ' ' (6), '🦀' (7), ' ' (8), 'C' (9)...
    assert_eq!(matches[0].range, 2..6);
    assert_eq!(matches[1].range, 9..13);

    // Replace next
    let rep_range = FindReplaceEngine::replace_next(&mut buffer, "Crab", "Lobster", 0, &options).unwrap();
    assert_eq!(rep_range, 2..9);
    assert_eq!(buffer.text(), "🦀 Lobster 🦀 Crab 🦀");

    // Replace all remaining
    let count = FindReplaceEngine::replace_all(&mut buffer, "Crab", "Shrimp", &options);
    assert_eq!(count, 1);
    assert_eq!(buffer.text(), "🦀 Lobster 🦀 Shrimp 🦀");
}

#[test]
fn test_empty_query_and_buffer() {
    let empty_buffer = TextBuffer::new();
    let options = FindOptions::new();

    assert!(FindReplaceEngine::find_all(&empty_buffer, "test", &options).is_empty());
    assert!(FindReplaceEngine::find_next(&empty_buffer, "test", 0, &options).is_none());
    assert!(FindReplaceEngine::find_prev(&empty_buffer, "test", 0, &options).is_none());

    let mut buffer = TextBuffer::from_str("some content");
    assert!(FindReplaceEngine::find_all(&buffer, "", &options).is_empty());
    assert!(FindReplaceEngine::find_next(&buffer, "", 0, &options).is_none());
    assert!(FindReplaceEngine::find_prev(&buffer, "", 0, &options).is_none());
    assert_eq!(FindReplaceEngine::replace_all(&mut buffer, "", "replacement", &options), 0);
    assert_eq!(buffer.text(), "some content");
}
