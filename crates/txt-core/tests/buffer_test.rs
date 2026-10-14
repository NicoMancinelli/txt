use txt_core::buffer::TextBuffer;
use txt_core::cursor::{CursorPosition, Selection};

#[test]
fn test_buffer_insert_and_delete() {
    let mut buf = TextBuffer::from_str("Hello World");
    assert_eq!(buf.len_chars(), 11);
    assert_eq!(buf.len_lines(), 1);

    buf.insert(5, ", Beautiful");
    assert_eq!(buf.text(), "Hello, Beautiful World");

    buf.delete(5..16);
    assert_eq!(buf.text(), "Hello World");
}

#[test]
fn test_multiline_indexing() {
    let text = "Line 1\nLine 2\nLine 3\n";
    let buf = TextBuffer::from_str(text);

    assert_eq!(buf.len_lines(), 4); // 3 lines + empty line after trailing newline

    let pos = buf.char_to_pos(7);
    assert_eq!(pos, CursorPosition::new(1, 0)); // Start of Line 2

    let offset = buf.pos_to_char(CursorPosition::new(1, 2));
    assert_eq!(offset, 9); // 'n' in Line 2
}

#[test]
fn test_multi_cursor_insertion() {
    let mut buf = TextBuffer::from_str("apple\nbanana\ncherry\n");
    // Place cursors at start of each line
    let sel1 = Selection::point(0);
    let sel2 = Selection::point(6);
    let sel3 = Selection::point(13);

    buf.insert_at_selections(&[sel1, sel2, sel3], "- ");
    assert_eq!(buf.text(), "- apple\n- banana\n- cherry\n");
}

#[test]
fn test_undo_redo() {
    let mut buf = TextBuffer::from_str("Initial");
    buf.insert(7, " Text");
    assert_eq!(buf.text(), "Initial Text");

    assert!(buf.undo());
    assert_eq!(buf.text(), "Initial");

    assert!(buf.redo());
    assert_eq!(buf.text(), "Initial Text");
}
