use txt_core::buffer::TextBuffer;
use txt_core::cursor::{
    add_cursor_above, add_cursor_below, is_word_char, select_all_occurrences,
    select_next_occurrence, select_word_at, split_selection_into_lines, Selection, SelectionSet,
};

#[test]
fn test_word_char_classification() {
    assert!(is_word_char('a'));
    assert!(is_word_char('Z'));
    assert!(is_word_char('9'));
    assert!(is_word_char('_'));
    assert!(is_word_char('α')); // Unicode Greek
    assert!(is_word_char('é')); // Unicode accented

    assert!(!is_word_char(' '));
    assert!(!is_word_char('\t'));
    assert!(!is_word_char('\n'));
    assert!(!is_word_char('+'));
    assert!(!is_word_char(';'));
    assert!(!is_word_char('('));
    assert!(!is_word_char('.'));
}

#[test]
fn test_word_boundary_selection_at_point() {
    let empty_buf = TextBuffer::new();
    assert_eq!(select_word_at(&empty_buf, 0), Selection::point(0));
    assert_eq!(select_word_at(&empty_buf, 10), Selection::point(0));

    let buf = TextBuffer::from_str("let mut my_variable_123 = true;");
    // Indices:
    // "let" -> 0..3
    // " " -> 3
    // "mut" -> 4..7
    // " " -> 7
    // "my_variable_123" -> 8..23
    // " " -> 23
    // "=" -> 24
    // " " -> 25
    // "true" -> 26..30
    // ";" -> 30

    // Cursor at start of "let"
    assert_eq!(select_word_at(&buf, 0), Selection::range(0, 3));
    // Cursor in middle of "let"
    assert_eq!(select_word_at(&buf, 1), Selection::range(0, 3));
    // Cursor at end of "let" (char 3 is space, preceded by 't')
    assert_eq!(select_word_at(&buf, 3), Selection::range(0, 3));

    // Cursor at start of "mut"
    assert_eq!(select_word_at(&buf, 4), Selection::range(4, 7));

    // Cursor in "my_variable_123"
    assert_eq!(select_word_at(&buf, 8), Selection::range(8, 23));
    assert_eq!(select_word_at(&buf, 11), Selection::range(8, 23)); // on underscore
    assert_eq!(select_word_at(&buf, 20), Selection::range(8, 23)); // on digits
    assert_eq!(select_word_at(&buf, 23), Selection::range(8, 23)); // right at end

    // Cursor on '=' (not a word char, preceded by space)
    assert_eq!(select_word_at(&buf, 24), Selection::point(24));

    // Cursor on "true"
    assert_eq!(select_word_at(&buf, 26), Selection::range(26, 30));
    assert_eq!(select_word_at(&buf, 30), Selection::range(26, 30)); // at ';' preceded by 'e'

    // Unicode test
    let uni_buf = TextBuffer::from_str("café crème_42");
    assert_eq!(select_word_at(&uni_buf, 0), Selection::range(0, 4));
    assert_eq!(select_word_at(&uni_buf, 3), Selection::range(0, 4));
    assert_eq!(select_word_at(&uni_buf, 5), Selection::range(5, 13));
}

#[test]
fn test_cmd_d_incremental_next_occurrence_and_wraparound() {
    let buf = TextBuffer::from_str("apple orange apple banana apple");
    // "apple" at:
    // 1: 0..5
    // 2: 13..18
    // 3: 26..31

    let mut selections = SelectionSet::from_selection(Selection::point(0));

    // 1. Initial Cmd+D: expands point cursor to word under cursor ("apple" at 0..5)
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 1);
    assert_eq!(selections.primary(), Selection::range(0, 5));
    assert_eq!(buf.slice_text(0..5), "apple");

    // 2. Second Cmd+D: selects next occurrence at 13..18
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 2);
    assert_eq!(selections.selections(), &[
        Selection::range(0, 5),
        Selection::range(13, 18),
    ]);
    assert_eq!(selections.primary(), Selection::range(13, 18));

    // 3. Third Cmd+D: selects third occurrence at 26..31
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
    assert_eq!(selections.selections(), &[
        Selection::range(0, 5),
        Selection::range(13, 18),
        Selection::range(26, 31),
    ]);
    assert_eq!(selections.primary(), Selection::range(26, 31));

    // 4. Fourth Cmd+D: all occurrences already selected -> wrap around, no duplicates
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
}

#[test]
fn test_cmd_d_wraparound_starting_from_middle() {
    let buf = TextBuffer::from_str("foo bar foo baz foo");
    // "foo" at 0..3, 8..11, 16..19

    // Start with cursor on middle "foo" (index 8)
    let mut selections = SelectionSet::from_selection(Selection::point(8));

    // 1. First Cmd+D: expands to 8..11
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 1);
    assert_eq!(selections.primary(), Selection::range(8, 11));

    // 2. Second Cmd+D: searches forward, finds 16..19
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 2);
    assert_eq!(selections.selections(), &[
        Selection::range(8, 11),
        Selection::range(16, 19),
    ]);
    assert_eq!(selections.primary(), Selection::range(16, 19));

    // 3. Third Cmd+D: hits EOF, wraps around to 0, selects 0..3
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
    assert_eq!(selections.selections(), &[
        Selection::range(0, 3),
        Selection::range(8, 11),
        Selection::range(16, 19),
    ]);
    assert_eq!(selections.primary(), Selection::range(0, 3));

    // 4. Fourth Cmd+D: all occurrences selected -> no change
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
}

#[test]
fn test_cmd_d_arbitrary_substring_selection() {
    let buf = TextBuffer::from_str("cat concatenate scatter cat");
    // "cat" occurs at:
    // 0..3 (cat)
    // 7..10 (con[cat]enate)
    // 18..21 (s[cat]ter)
    // 24..27 (cat)

    // Start with explicit range inside "concatenate" (7..10)
    let mut selections = SelectionSet::from_selection(Selection::range(7, 10));

    // Cmd+D -> finds scatter (17..20)
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 2);
    assert_eq!(selections.primary(), Selection::range(17, 20));

    // Cmd+D -> finds last cat (24..27)
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
    assert_eq!(selections.primary(), Selection::range(24, 27));

    // Cmd+D -> wraps around, finds first cat (0..3)
    select_next_occurrence(&buf, &mut selections);
    assert_eq!(selections.len(), 4);
    assert_eq!(selections.selections(), &[
        Selection::range(0, 3),
        Selection::range(7, 10),
        Selection::range(17, 20),
        Selection::range(24, 27),
    ]);
}

#[test]
fn test_cmd_ctrl_g_select_all_occurrences() {
    let buf = TextBuffer::from_str("key = 1;\nkey = 2;\nkey = 3;\nkey = 4;");
    let mut selections = SelectionSet::from_selection(Selection::point(0));

    select_all_occurrences(&buf, &mut selections);

    assert_eq!(selections.len(), 4);
    for sel in selections.selections() {
        assert_eq!(buf.slice_text(sel.min_char()..sel.max_char()), "key");
    }

    // Explicit range select all
    let buf2 = TextBuffer::from_str("(id) -> 1, (id) -> 2, (id) -> 3");
    let mut selections2 = SelectionSet::from_selection(Selection::range(0, 4)); // "(id)"
    select_all_occurrences(&buf2, &mut selections2);
    assert_eq!(selections2.len(), 3);
    for sel in selections2.selections() {
        assert_eq!(buf2.slice_text(sel.min_char()..sel.max_char()), "(id)");
    }
}

#[test]
fn test_cmd_shift_l_split_multi_line_selection() {
    let text = "first line\nsecond line\nthird line\nfourth line";
    let buf = TextBuffer::from_str(text);

    // Spanning from "line" in first line (char 6) to "line" in third line (char 29)
    // "first line\n" (0..11)
    // "second line\n" (11..23)
    // "third line\n" (23..34)
    let mut selections = SelectionSet::from_selection(Selection::range(6, 28));

    split_selection_into_lines(&buf, &mut selections);

    assert_eq!(selections.len(), 3);
    // Line 0: char 6 to line content end (10) -> "line"
    assert_eq!(buf.slice_text(selections.selections()[0].min_char()..selections.selections()[0].max_char()), "line");
    // Line 1: char 11 to 22 -> "second line"
    assert_eq!(buf.slice_text(selections.selections()[1].min_char()..selections.selections()[1].max_char()), "second line");
    // Line 2: char 23 to 28 -> "third"
    assert_eq!(buf.slice_text(selections.selections()[2].min_char()..selections.selections()[2].max_char()), "third");

    // Test selection ending exactly at column 0 of next line (trailing newline selected)
    // Selects "first line\nsecond line\n" (0..23) where char 23 is start of "third line"
    let mut full_line_sel = SelectionSet::from_selection(Selection::range(0, 23));
    split_selection_into_lines(&buf, &mut full_line_sel);
    assert_eq!(full_line_sel.len(), 2);
    assert_eq!(buf.slice_text(full_line_sel.selections()[0].min_char()..full_line_sel.selections()[0].max_char()), "first line");
    assert_eq!(buf.slice_text(full_line_sel.selections()[1].min_char()..full_line_sel.selections()[1].max_char()), "second line");

    // Single-line selection should remain unaffected
    let mut single_sel = SelectionSet::from_selection(Selection::range(0, 5));
    split_selection_into_lines(&buf, &mut single_sel);
    assert_eq!(single_sel.len(), 1);
    assert_eq!(single_sel.primary(), Selection::range(0, 5));
}

#[test]
fn test_add_cursor_above_and_below() {
    let text = "aaaa\nbbbb\ncccc\ndddd";
    let buf = TextBuffer::from_str(text);

    // Start at line 2 col 2 ('c' at offset 12)
    // line 0: "aaaa\n" (0..5)
    // line 1: "bbbb\n" (5..10)
    // line 2: "cccc\n" (10..15), col 2 is offset 12
    // line 3: "dddd" (15..19), col 2 is offset 17
    let mut selections = SelectionSet::from_selection(Selection::point(12));

    // Add cursor above -> adds at line 1 col 2 (offset 7)
    add_cursor_above(&buf, &mut selections);
    assert_eq!(selections.len(), 2);
    assert_eq!(selections.selections(), &[Selection::point(7), Selection::point(12)]);

    // Add cursor above again -> adds at line 0 col 2 (offset 2)
    add_cursor_above(&buf, &mut selections);
    assert_eq!(selections.len(), 3);
    assert_eq!(selections.selections(), &[
        Selection::point(2),
        Selection::point(7),
        Selection::point(12),
    ]);

    // Add cursor above at line 0 -> boundary guard prevents out of bounds
    add_cursor_above(&buf, &mut selections);
    assert_eq!(selections.len(), 3);

    // Test add cursor below
    let mut below_selections = SelectionSet::from_selection(Selection::point(7)); // line 1 col 2
    add_cursor_below(&buf, &mut below_selections);
    assert_eq!(below_selections.len(), 2);
    assert_eq!(below_selections.selections(), &[Selection::point(7), Selection::point(12)]);

    add_cursor_below(&buf, &mut below_selections);
    assert_eq!(below_selections.len(), 3);
    assert_eq!(below_selections.selections(), &[
        Selection::point(7),
        Selection::point(12),
        Selection::point(17),
    ]);

    // Bottom of buffer boundary check
    add_cursor_below(&buf, &mut below_selections);
    assert_eq!(below_selections.len(), 3);
}

#[test]
fn test_add_cursor_above_below_jagged_line_clamping() {
    let text = "long line here\nhi\nanother long line";
    let buf = TextBuffer::from_str(text);

    // Cursor on line 0, col 8 ('i')
    let mut selections = SelectionSet::from_selection(Selection::point(8));

    // Line 1 is only "hi" (len 2). Column 8 clamps to column 2 (offset 17)
    add_cursor_below(&buf, &mut selections);
    assert_eq!(selections.len(), 2);
    assert_eq!(selections.selections()[1], Selection::point(17)); // end of "hi"
}

#[test]
fn test_multi_cursor_simultaneous_editing_integration() {
    let mut buf = TextBuffer::from_str("foo = 1;\nfoo = 2;\nfoo = 3;");
    let mut selections = SelectionSet::from_selection(Selection::point(0));

    // Select all occurrences of "foo"
    select_all_occurrences(&buf, &mut selections);
    assert_eq!(selections.len(), 3);

    // Perform multi-cursor simultaneous replacement
    let new_selections = buf.insert_at_selections(selections.selections(), "variable_name");
    assert_eq!(new_selections.len(), 3);
    assert_eq!(buf.text(), "variable_name = 1;\nvariable_name = 2;\nvariable_name = 3;");
}
