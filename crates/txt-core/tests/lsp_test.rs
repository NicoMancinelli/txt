use std::path::{Path, PathBuf};
use txt_core::cursor::CursorPosition;
use txt_core::lsp::{
    CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity, HoverInfo,
    LspClientState, LspFramingError, LspFramingParser, Position, Range, RpcId, RpcNotification,
    RpcResponse,
};

#[test]
fn test_lsp_position_and_range() {
    let pos = Position::new(5, 10);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.character, 10);

    // Conversion with CursorPosition
    let cursor_pos = CursorPosition::new(5, 10);
    let converted_pos: Position = cursor_pos.into();
    assert_eq!(converted_pos, pos);

    let converted_cursor: CursorPosition = pos.into();
    assert_eq!(converted_cursor, cursor_pos);

    let range = Range::new(Position::new(2, 0), Position::new(4, 20));
    assert!(range.contains(&Position::new(3, 5)));
    assert!(range.contains(&Position::new(2, 0)));
    assert!(range.contains(&Position::new(4, 20)));
    assert!(!range.contains(&Position::new(1, 10)));
    assert!(!range.contains(&Position::new(5, 0)));
    assert!(!range.contains(&Position::new(4, 21)));
}

#[test]
fn test_diagnostic_serialization() {
    let diag = Diagnostic::new(
        Range::new(Position::new(0, 0), Position::new(0, 10)),
        "syntax error: missing semicolon",
    )
    .with_severity(DiagnosticSeverity::Error)
    .with_code("E0001")
    .with_source("rustc");

    let json = serde_json::to_string(&diag).unwrap();
    assert!(json.contains("\"severity\":1"));
    assert!(json.contains("\"code\":\"E0001\""));
    assert!(json.contains("\"source\":\"rustc\""));

    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, diag);
    assert_eq!(deserialized.severity, Some(DiagnosticSeverity::Error));
}

#[test]
fn test_completion_item_serialization() {
    let item = CompletionItem::new("my_function")
        .with_kind(CompletionItemKind::Function)
        .with_detail("fn() -> bool")
        .with_insert_text("my_function()")
        .with_documentation("A helpful function");

    assert_eq!(item.effective_insert_text(), "my_function()");

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"kind\":3")); // Function = 3

    let deserialized: CompletionItem = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, item);
    assert_eq!(deserialized.kind, Some(CompletionItemKind::Function));
}

#[test]
fn test_hover_info_creation() {
    let hover = HoverInfo::new("```rust\nfn calculate(x: i32) -> i32\n```")
        .with_range(Range::point(Position::new(10, 4)));

    assert_eq!(hover.contents, "```rust\nfn calculate(x: i32) -> i32\n```");
    assert!(hover.range.is_some());

    let json = serde_json::to_string(&hover).unwrap();
    let deserialized: HoverInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, hover);
}

#[test]
fn test_lsp_framing_parser_single_frame() {
    let mut parser = LspFramingParser::new();
    let payload = r#"{"jsonrpc":"2.0","id":1,"method":"initialized","params":{}}"#;
    let framed = LspFramingParser::encode_frame(payload);

    parser.feed(&framed);
    let parsed = parser.poll_message().unwrap();
    assert_eq!(parsed, Some(payload.to_string()));

    // Next poll should be empty
    assert_eq!(parser.poll_message().unwrap(), None);
}

#[test]
fn test_lsp_framing_parser_multiple_frames() {
    let mut parser = LspFramingParser::new();
    let msg1 = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
    let msg2 = r#"{"jsonrpc":"2.0","id":2,"result":{"capabilities":{}}}"#;

    let mut stream = LspFramingParser::encode_frame(msg1);
    stream.extend(LspFramingParser::encode_frame(msg2));

    parser.feed(&stream);

    let first = parser.poll_message().unwrap();
    assert_eq!(first, Some(msg1.to_string()));

    let second = parser.poll_message().unwrap();
    assert_eq!(second, Some(msg2.to_string()));

    assert_eq!(parser.poll_message().unwrap(), None);
}

#[test]
fn test_lsp_framing_parser_chunked_streaming() {
    let mut parser = LspFramingParser::new();
    let msg = r#"{"jsonrpc":"2.0","method":"window/showMessage","params":{"type":1,"message":"test"}}"#;
    let framed = LspFramingParser::encode_frame(msg);

    // Feed in small 5-byte chunks
    for chunk in framed.chunks(5) {
        parser.feed(chunk);
    }

    let parsed = parser.poll_message().unwrap();
    assert_eq!(parsed, Some(msg.to_string()));
    assert_eq!(parser.poll_message().unwrap(), None);
}

#[test]
fn test_lsp_framing_parser_additional_headers() {
    let mut parser = LspFramingParser::new();
    let payload = r#"{"jsonrpc":"2.0","id":100}"#;
    let header = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{}",
        payload.len(),
        payload
    );

    parser.feed_str(&header);
    let parsed = parser.poll_message().unwrap();
    assert_eq!(parsed, Some(payload.to_string()));
}

#[test]
fn test_lsp_framing_parser_errors() {
    let mut parser = LspFramingParser::new();

    // Missing Content-Length header
    parser.feed_str("Invalid-Header: value\r\n\r\n{}");
    assert_eq!(
        parser.poll_message().unwrap_err(),
        LspFramingError::MissingContentLength
    );

    parser.clear();
    // Invalid Content-Length number
    parser.feed_str("Content-Length: not-a-number\r\n\r\n{}");
    match parser.poll_message() {
        Err(LspFramingError::InvalidContentLength(_)) => {}
        other => panic!("expected InvalidContentLength, got {other:?}"),
    }
}

#[test]
fn test_lsp_client_state_requests_and_responses() {
    let state = LspClientState::new();
    assert_eq!(state.pending_count(), 0);

    let req = state.create_request("workspace/symbol", Some(serde_json::json!({"query": "main"})));
    assert_eq!(req.method, "workspace/symbol");
    assert_eq!(req.id, RpcId::Number(1));
    assert_eq!(state.pending_count(), 1);
    assert!(state.is_request_pending(&req.id));

    // Handle incoming response
    let response = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(RpcId::Number(1)),
        result: Some(serde_json::json!([])),
        error: None,
    };

    let handled = state.handle_response(&response).unwrap();
    assert!(handled.is_some());
    assert_eq!(handled.unwrap().method, "workspace/symbol");
    assert_eq!(state.pending_count(), 0);
    assert!(!state.is_request_pending(&req.id));
}

#[test]
fn test_lsp_client_state_diagnostics_management() {
    let state = LspClientState::new();
    let file = PathBuf::from("/workspace/src/lib.rs");

    let diag1 = Diagnostic::new(Range::point(Position::new(10, 5)), "type mismatch")
        .with_severity(DiagnosticSeverity::Error);
    let diag2 = Diagnostic::new(Range::point(Position::new(15, 2)), "unused variable")
        .with_severity(DiagnosticSeverity::Warning);

    state.publish_diagnostics(file.clone(), vec![diag1.clone(), diag2.clone()]);

    let diags = state.get_diagnostics(&file);
    assert_eq!(diags.len(), 2);
    assert_eq!(state.error_count(&file), 1);
    assert_eq!(state.warning_count(&file), 1);

    let line_10_diags = state.diagnostics_for_line(&file, 10);
    assert_eq!(line_10_diags.len(), 1);
    assert_eq!(line_10_diags[0].message, "type mismatch");

    // Notification handling
    let notification = RpcNotification::new(
        "textDocument/publishDiagnostics",
        Some(serde_json::json!({
            "uri": "file:///workspace/src/lib.rs",
            "diagnostics": [
                {
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 5 } },
                    "severity": 1,
                    "message": "syntax error"
                }
            ]
        })),
    );

    let updated_path = state.handle_publish_diagnostics(&notification).unwrap();
    assert_eq!(updated_path, PathBuf::from("/workspace/src/lib.rs"));
    assert_eq!(state.get_diagnostics(&file).len(), 1);
    assert_eq!(state.get_diagnostics(&file)[0].message, "syntax error");

    state.clear_diagnostics(&file);
    assert_eq!(state.get_diagnostics(&file).len(), 0);
}

#[test]
fn test_lsp_completion_and_hover_parsing() {
    let state = LspClientState::new();
    let file = Path::new("/workspace/src/main.rs");
    let pos = Position::new(20, 8);

    // Request completion
    let req = state.request_completion(file, pos, "str");
    assert_eq!(req.method, "textDocument/completion");
    assert!(state.is_request_pending(&req.id));

    // Response with CompletionList
    let response = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(req.id.clone()),
        result: Some(serde_json::json!({
            "isIncomplete": false,
            "items": [
                { "label": "stream", "kind": 6, "detail": "stream iterator" },
                { "label": "string", "kind": 14, "insertText": "String::from()" }
            ]
        })),
        error: None,
    };

    let items = state.parse_completion_response(&response).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].label, "stream");
    assert_eq!(items[1].effective_insert_text(), "String::from()");

    // Request and parse hover
    let hover_req = state.request_hover(file, pos);
    assert_eq!(hover_req.method, "textDocument/hover");

    let hover_resp = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(hover_req.id),
        result: Some(serde_json::json!({
            "contents": {
                "kind": "markdown",
                "value": "# Documentation\n\nReturns standard stream."
            },
            "range": {
                "start": { "line": 20, "character": 8 },
                "end": { "line": 20, "character": 14 }
            }
        })),
        error: None,
    };

    let hover_info = state.parse_hover_response(&hover_resp).unwrap().unwrap();
    assert!(hover_info.contents.contains("Returns standard stream."));
    assert_eq!(
        hover_info.range,
        Some(Range::new(Position::new(20, 8), Position::new(20, 14)))
    );
}
