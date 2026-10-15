use std::path::PathBuf;
use txt_core::buffer::TextBuffer;
use txt_core::symbols::{DocumentSymbol, SymbolExtractor, SymbolKind};

#[test]
fn test_symbol_kind_properties() {
    assert_eq!(SymbolKind::Function.as_str(), "Function");
    assert_eq!(SymbolKind::Struct.as_str(), "Struct");
    assert_eq!(SymbolKind::Enum.as_str(), "Enum");
    assert_eq!(SymbolKind::Trait.as_str(), "Trait");
    assert_eq!(SymbolKind::Class.as_str(), "Class");
    assert_eq!(SymbolKind::Heading.as_str(), "Heading");
    assert_eq!(SymbolKind::Constant.as_str(), "Constant");
    assert_eq!(SymbolKind::Variable.as_str(), "Variable");

    assert_eq!(format!("{}", SymbolKind::Function), "Function");
    assert_eq!(SymbolKind::Function.icon(), "ƒ");
}

#[test]
fn test_document_symbol_builder_and_serde() {
    let sym = DocumentSymbol::new("calculate", SymbolKind::Function, 10, 4)
        .with_detail("fn calculate(x: i32) -> i32");

    assert_eq!(sym.name, "calculate");
    assert_eq!(sym.kind, SymbolKind::Function);
    assert_eq!(sym.line, 10);
    assert_eq!(sym.column, 4);
    assert_eq!(sym.detail.as_deref(), Some("fn calculate(x: i32) -> i32"));

    // Serde roundtrip
    let json = serde_json::to_string(&sym).expect("serialization failed");
    let deserialized: DocumentSymbol = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(sym, deserialized);
}

#[test]
fn test_rust_symbol_extraction() {
    let code = r#"
// This is a comment: fn ignored_comment() {}
pub fn process_data(input: &str) -> usize {
    42
}

pub(crate) async fn fetch_remote() -> Result<(), ()> {
    Ok(())
}

unsafe fn dangerous_op() {}

pub const fn const_calc() -> usize { 1 }

pub struct BufferEngine {
    capacity: usize,
}

pub enum LineEnding {
    Lf,
    Crlf,
}

pub trait Synchronizer {
    fn sync(&self);
}

impl BufferEngine {
    pub fn new() -> Self { Self { capacity: 0 } }
}

impl<T> Synchronizer for BufferEngine {
    fn sync(&self) {}
}

pub const MAX_CAPACITY: usize = 1024 * 1024;
static GLOBAL_COUNTER: usize = 0;
"#;

    let buf = TextBuffer::from_str(code);
    let symbols = SymbolExtractor::extract_symbols(&buf, "rust");

    let names: Vec<(&str, SymbolKind)> = symbols.iter().map(|s| (s.name.as_str(), s.kind)).collect();

    assert!(names.contains(&("process_data", SymbolKind::Function)));
    assert!(names.contains(&("fetch_remote", SymbolKind::Function)));
    assert!(names.contains(&("dangerous_op", SymbolKind::Function)));
    assert!(names.contains(&("const_calc", SymbolKind::Function)));
    assert!(names.contains(&("BufferEngine", SymbolKind::Struct)));
    assert!(names.contains(&("LineEnding", SymbolKind::Enum)));
    assert!(names.contains(&("Synchronizer", SymbolKind::Trait)));
    assert!(names.contains(&("BufferEngine", SymbolKind::Class)));
    assert!(names.contains(&("Synchronizer for BufferEngine", SymbolKind::Trait)));
    assert!(names.contains(&("MAX_CAPACITY", SymbolKind::Constant)));
    assert!(names.contains(&("GLOBAL_COUNTER", SymbolKind::Variable)));

    // Verify comment was ignored
    assert!(!names.iter().any(|(n, _)| *n == "ignored_comment"));

    // Check detail and position
    let proc_sym = symbols.iter().find(|s| s.name == "process_data").unwrap();
    assert!(proc_sym.column > 0);
    assert!(proc_sym.detail.as_ref().unwrap().contains("fn process_data"));
}

#[test]
fn test_python_symbol_extraction() {
    let code = r#"
# Comment: def ignored()
def calculate_metrics(data):
    return len(data)

async def stream_events():
    pass

class DataPipeline(BasePipeline):
    def __init__(self):
        self.count = 0

    def run(self):
        pass

MAX_CONNECTIONS = 100
current_status = "idle"
"#;

    let buf = TextBuffer::from_str(code);
    let symbols = SymbolExtractor::extract_symbols(&buf, "python");

    let names: Vec<(&str, SymbolKind)> = symbols.iter().map(|s| (s.name.as_str(), s.kind)).collect();

    assert!(names.contains(&("calculate_metrics", SymbolKind::Function)));
    assert!(names.contains(&("stream_events", SymbolKind::Function)));
    assert!(names.contains(&("DataPipeline", SymbolKind::Class)));
    assert!(names.contains(&("__init__", SymbolKind::Function)));
    assert!(names.contains(&("run", SymbolKind::Function)));
    assert!(names.contains(&("MAX_CONNECTIONS", SymbolKind::Constant)));
    assert!(names.contains(&("current_status", SymbolKind::Variable)));
    assert!(!names.iter().any(|(n, _)| *n == "ignored"));
}

#[test]
fn test_markdown_symbol_extraction() {
    let doc = r#"
# System Architecture

Introduction to the system.

## Performance Invariants
Details on invariants.

### Zero Runtime Bloat
No chromium.

### O(log N) Buffer Edits
Rope structure.

#### Deep Leaf Node
Extra details.
"#;

    let buf = TextBuffer::from_str(doc);
    let symbols = SymbolExtractor::extract_symbols(&buf, "markdown");

    assert_eq!(symbols.len(), 5);
    assert_eq!(symbols[0].name, "System Architecture");
    assert_eq!(symbols[0].kind, SymbolKind::Heading);
    assert_eq!(symbols[0].detail.as_deref(), Some("H1"));

    assert_eq!(symbols[1].name, "Performance Invariants");
    assert_eq!(symbols[1].kind, SymbolKind::Heading);
    assert_eq!(symbols[1].detail.as_deref(), Some("H2"));

    assert_eq!(symbols[2].name, "Zero Runtime Bloat");
    assert_eq!(symbols[2].kind, SymbolKind::Heading);
    assert_eq!(symbols[2].detail.as_deref(), Some("H3"));

    assert_eq!(symbols[3].name, "O(log N) Buffer Edits");
    assert_eq!(symbols[3].kind, SymbolKind::Heading);
    assert_eq!(symbols[3].detail.as_deref(), Some("H3"));

    assert_eq!(symbols[4].name, "Deep Leaf Node");
    assert_eq!(symbols[4].kind, SymbolKind::Heading);
    assert_eq!(symbols[4].detail.as_deref(), Some("H4"));
}

#[test]
fn test_javascript_typescript_symbol_extraction() {
    let js = r#"
// Comment: function ignored() {}
export async function initializeApp(config) {
    return true;
}

function helperUtil() {
    return 1;
}

export class EditorView extends Component {
    render() {}
}

const handleClick = (event) => {
    console.log(event);
};

export const fetchData = async (url) => {
    return fetch(url);
};

let activeTabId = 42;
const MAX_BUFFER_SIZE = 1048576;
var legacyFlag = true;
"#;

    let buf = TextBuffer::from_str(js);
    let symbols = SymbolExtractor::extract_symbols(&buf, "typescript");

    let names: Vec<(&str, SymbolKind)> = symbols.iter().map(|s| (s.name.as_str(), s.kind)).collect();

    assert!(names.contains(&("initializeApp", SymbolKind::Function)));
    assert!(names.contains(&("helperUtil", SymbolKind::Function)));
    assert!(names.contains(&("EditorView", SymbolKind::Class)));
    assert!(names.contains(&("handleClick", SymbolKind::Function)));
    assert!(names.contains(&("fetchData", SymbolKind::Function)));
    assert!(names.contains(&("activeTabId", SymbolKind::Variable)));
    assert!(names.contains(&("MAX_BUFFER_SIZE", SymbolKind::Constant)));
    assert!(names.contains(&("legacyFlag", SymbolKind::Variable)));
    assert!(!names.iter().any(|(n, _)| *n == "ignored"));
}

#[test]
fn test_language_detection_and_file_extension() {
    let mut buf = TextBuffer::from_str("fn rust_func() {}\n");
    buf.set_file_path(PathBuf::from("/path/to/main.rs"));

    // Test with explicit extension
    let syms1 = SymbolExtractor::extract_symbols(&buf, "rs");
    assert_eq!(syms1.len(), 1);
    assert_eq!(syms1[0].name, "rust_func");

    // Test with auto-detection via file_path
    let syms2 = SymbolExtractor::extract_symbols(&buf, "auto");
    assert_eq!(syms2.len(), 1);
    assert_eq!(syms2[0].name, "rust_func");

    // Test empty language string falls back to file path
    let syms3 = SymbolExtractor::extract_symbols(&buf, "");
    assert_eq!(syms3.len(), 1);
    assert_eq!(syms3[0].name, "rust_func");
}
