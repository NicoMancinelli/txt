use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Instant;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use crate::cursor::CursorPosition;

/// 0-based position in a text document (line and character offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

impl From<CursorPosition> for Position {
    fn from(pos: CursorPosition) -> Self {
        Self {
            line: pos.line as u32,
            character: pos.column as u32,
        }
    }
}

impl From<Position> for CursorPosition {
    fn from(pos: Position) -> Self {
        CursorPosition::new(pos.line as usize, pos.character as usize)
    }
}

/// A text range expressed as a start and end Position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn point(pos: Position) -> Self {
        Self { start: pos, end: pos }
    }

    /// Checks if a position falls within this range (inclusive).
    pub fn contains(&self, pos: &Position) -> bool {
        if pos.line < self.start.line || pos.line > self.end.line {
            return false;
        }
        if pos.line == self.start.line && pos.character < self.start.character {
            return false;
        }
        if pos.line == self.end.line && pos.character > self.end.character {
            return false;
        }
        true
    }
}

/// Diagnostic severity levels matching LSP wire integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.as_u32())
    }
}

impl<'de> Deserialize<'de> for DiagnosticSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = u32::deserialize(deserializer)?;
        match val {
            1 => Ok(DiagnosticSeverity::Error),
            2 => Ok(DiagnosticSeverity::Warning),
            3 => Ok(DiagnosticSeverity::Information),
            4 => Ok(DiagnosticSeverity::Hint),
            other => Err(serde::de::Error::custom(format!("invalid DiagnosticSeverity: {other}"))),
        }
    }
}

/// A diagnostic representing a compiler or linter issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub message: String,
}

impl Diagnostic {
    pub fn new(range: Range, message: impl Into<String>) -> Self {
        Self {
            range,
            severity: None,
            code: None,
            source: None,
            message: message.into(),
        }
    }

    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Completion item kinds per the LSP specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

impl CompletionItemKind {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl Serialize for CompletionItemKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.as_u32())
    }
}

impl<'de> Deserialize<'de> for CompletionItemKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = u32::deserialize(deserializer)?;
        match val {
            1 => Ok(CompletionItemKind::Text),
            2 => Ok(CompletionItemKind::Method),
            3 => Ok(CompletionItemKind::Function),
            4 => Ok(CompletionItemKind::Constructor),
            5 => Ok(CompletionItemKind::Field),
            6 => Ok(CompletionItemKind::Variable),
            7 => Ok(CompletionItemKind::Class),
            8 => Ok(CompletionItemKind::Interface),
            9 => Ok(CompletionItemKind::Module),
            10 => Ok(CompletionItemKind::Property),
            11 => Ok(CompletionItemKind::Unit),
            12 => Ok(CompletionItemKind::Value),
            13 => Ok(CompletionItemKind::Enum),
            14 => Ok(CompletionItemKind::Keyword),
            15 => Ok(CompletionItemKind::Snippet),
            16 => Ok(CompletionItemKind::Color),
            17 => Ok(CompletionItemKind::File),
            18 => Ok(CompletionItemKind::Reference),
            19 => Ok(CompletionItemKind::Folder),
            20 => Ok(CompletionItemKind::EnumMember),
            21 => Ok(CompletionItemKind::Constant),
            22 => Ok(CompletionItemKind::Struct),
            23 => Ok(CompletionItemKind::Event),
            24 => Ok(CompletionItemKind::Operator),
            25 => Ok(CompletionItemKind::TypeParameter),
            other => Err(serde::de::Error::custom(format!("invalid CompletionItemKind: {other}"))),
        }
    }
}

/// An autocompletion proposal from a language server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CompletionItemKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "insert_text")]
    pub insert_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "sort_text")]
    pub sort_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "filter_text")]
    pub filter_text: Option<String>,
}

impl CompletionItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: None,
            detail: None,
            documentation: None,
            insert_text: None,
            sort_text: None,
            filter_text: None,
        }
    }

    pub fn with_kind(mut self, kind: CompletionItemKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    pub fn with_insert_text(mut self, insert_text: impl Into<String>) -> Self {
        self.insert_text = Some(insert_text.into());
        self
    }

    /// Resolves the actual text to be inserted into the buffer upon selection.
    pub fn effective_insert_text(&self) -> &str {
        self.insert_text.as_deref().unwrap_or(&self.label)
    }
}

/// Hover documentation and type information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverInfo {
    pub contents: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

impl HoverInfo {
    pub fn new(contents: impl Into<String>) -> Self {
        Self {
            contents: contents.into(),
            range: None,
        }
    }

    pub fn with_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC Wire Protocol Types
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 Request ID (can be numeric or string per spec).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    String(String),
}

impl std::fmt::Display for RpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcId::Number(n) => write!(f, "{n}"),
            RpcId::String(s) => write!(f, "\"{s}\""),
        }
    }
}

/// Outgoing or incoming JSON-RPC Request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: RpcId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl RpcRequest {
    pub fn new(id: RpcId, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC Error payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC Response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// JSON-RPC Notification payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl RpcNotification {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Unified JSON-RPC message container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcMessage {
    Request(RpcRequest),
    Response(RpcResponse),
    Notification(RpcNotification),
}

// ---------------------------------------------------------------------------
// LSP Stdio Framing Parser
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LspFramingError {
    #[error("Header contains invalid UTF-8")]
    InvalidHeaderUtf8,
    #[error("Missing Content-Length header")]
    MissingContentLength,
    #[error("Invalid Content-Length value: {0}")]
    InvalidContentLength(String),
    #[error("Body contains invalid UTF-8")]
    InvalidBodyUtf8,
}

/// Streaming parser for LSP messages framed with `Content-Length: ...\r\n\r\n`.
#[derive(Debug, Default)]
pub struct LspFramingParser {
    buffer: Vec<u8>,
}

impl LspFramingParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Appends incoming bytes to the internal parse buffer.
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Appends incoming string slice to the internal parse buffer.
    pub fn feed_str(&mut self, text: &str) {
        self.feed(text.as_bytes());
    }

    /// Attempts to extract the next complete framed message payload.
    /// Returns `Ok(Some(payload))` if a complete message was parsed,
    /// `Ok(None)` if more data is needed, or `Err(LspFramingError)` on malformed headers.
    pub fn poll_message(&mut self) -> Result<Option<String>, LspFramingError> {
        let delimiter = b"\r\n\r\n";
        let header_end = match self.buffer.windows(4).position(|w| w == delimiter) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let header_str = std::str::from_utf8(&self.buffer[..header_end])
            .map_err(|_| LspFramingError::InvalidHeaderUtf8)?;

        let mut content_length: Option<usize> = None;
        for line in header_str.lines() {
            let line = line.trim();
            if let Some((key, val)) = line.split_once(':') {
                if key.trim().eq_ignore_ascii_case("content-length") {
                    let len_str = val.trim();
                    let len = len_str
                        .parse::<usize>()
                        .map_err(|_| LspFramingError::InvalidContentLength(len_str.to_string()))?;
                    content_length = Some(len);
                }
            }
        }

        let content_length = content_length.ok_or(LspFramingError::MissingContentLength)?;
        let total_frame_len = header_end + 4 + content_length;

        if self.buffer.len() < total_frame_len {
            // Wait for more body bytes to arrive
            return Ok(None);
        }

        let body_bytes = &self.buffer[(header_end + 4)..total_frame_len];
        let body = std::str::from_utf8(body_bytes)
            .map_err(|_| LspFramingError::InvalidBodyUtf8)?
            .to_string();

        self.buffer.drain(..total_frame_len);
        Ok(Some(body))
    }

    /// Enforces standard LSP framing for an outgoing payload string.
    pub fn encode_frame(payload: &str) -> Vec<u8> {
        format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload).into_bytes()
    }

    /// Serializes a JSON-RPC message into an LSP stdio frame.
    pub fn encode_message<T: Serialize>(message: &T) -> Result<Vec<u8>, serde_json::Error> {
        let json = serde_json::to_string(message)?;
        Ok(Self::encode_frame(&json))
    }

    /// Clears any unprocessed bytes in the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the number of raw bytes currently in the buffer.
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

// ---------------------------------------------------------------------------
// URI and Path Utilities
// ---------------------------------------------------------------------------

pub fn path_to_uri(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with("file://") {
        s.to_string()
    } else {
        format!("file://{}", path.display())
    }
}

pub fn uri_to_path(uri: &str) -> PathBuf {
    let raw = if let Some(stripped) = uri.strip_prefix("file://") {
        stripped
    } else {
        uri
    };
    PathBuf::from(decode_uri_percent(raw))
}

fn decode_uri_percent(uri: &str) -> String {
    let mut result = String::with_capacity(uri.len());
    let mut chars = uri.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                let hex_str = format!("{h1}{h2}");
                if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                    result.push(byte as char);
                    continue;
                }
                result.push('%');
                result.push(h1);
                result.push(h2);
            } else {
                result.push('%');
                if let Some(h1) = h1 {
                    result.push(h1);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// LSP Client State Management
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("Framing error: {0}")]
    Framing(#[from] LspFramingError),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LSP RPC error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("Invalid protocol message: {0}")]
    InvalidMessage(String),
    #[error("Unknown request ID: {0:?}")]
    UnknownRequestId(RpcId),
}

/// Metadata stored for pending requests awaiting response from the language server.
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub id: RpcId,
    pub method: String,
    pub sent_at: Instant,
}

/// Client state tracker for an LSP server session.
/// Manages request IDs, pending in-flight requests, diagnostics caches, and autocompletion queries.
pub struct LspClientState {
    next_id: AtomicI64,
    pending_requests: RwLock<HashMap<RpcId, PendingRequest>>,
    diagnostics: RwLock<HashMap<PathBuf, Vec<Diagnostic>>>,
    active_completion_prefix: RwLock<HashMap<RpcId, String>>,
}

impl Default for LspClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl LspClientState {
    pub fn new() -> Self {
        Self {
            next_id: AtomicI64::new(1),
            pending_requests: RwLock::new(HashMap::new()),
            diagnostics: RwLock::new(HashMap::new()),
            active_completion_prefix: RwLock::new(HashMap::new()),
        }
    }

    /// Allocates the next unique numeric JSON-RPC request ID.
    pub fn next_request_id(&self) -> RpcId {
        RpcId::Number(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates and registers a new JSON-RPC request.
    pub fn create_request(&self, method: impl Into<String>, params: Option<serde_json::Value>) -> RpcRequest {
        let id = self.next_request_id();
        let method = method.into();
        let request = RpcRequest::new(id.clone(), method.clone(), params);

        self.pending_requests.write().insert(
            id.clone(),
            PendingRequest {
                id,
                method,
                sent_at: Instant::now(),
            },
        );

        request
    }

    /// Creates a JSON-RPC notification (no ID or response expected).
    pub fn create_notification(&self, method: impl Into<String>, params: Option<serde_json::Value>) -> RpcNotification {
        RpcNotification::new(method, params)
    }

    /// Handles a response received from the language server, matching it to its pending request.
    pub fn handle_response(&self, response: &RpcResponse) -> Result<Option<PendingRequest>, LspError> {
        if let Some(err) = &response.error {
            return Err(LspError::Rpc {
                code: err.code,
                message: err.message.clone(),
            });
        }

        if let Some(id) = &response.id {
            let pending = self.pending_requests.write().remove(id);
            self.active_completion_prefix.write().remove(id);
            Ok(pending)
        } else {
            Ok(None)
        }
    }

    /// Returns true if a request with the given ID is currently pending.
    pub fn is_request_pending(&self, id: &RpcId) -> bool {
        self.pending_requests.read().contains_key(id)
    }

    /// Returns the number of currently pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending_requests.read().len()
    }

    // -----------------------------------------------------------------------
    // Diagnostics Cache
    // -----------------------------------------------------------------------

    /// Updates cached diagnostics for a file path.
    pub fn publish_diagnostics(&self, path: impl Into<PathBuf>, diagnostics: Vec<Diagnostic>) {
        let mut cache = self.diagnostics.write();
        cache.insert(path.into(), diagnostics);
    }

    /// Processes an incoming `textDocument/publishDiagnostics` notification.
    pub fn handle_publish_diagnostics(&self, notification: &RpcNotification) -> Result<PathBuf, LspError> {
        if notification.method != "textDocument/publishDiagnostics" {
            return Err(LspError::InvalidMessage(format!(
                "expected textDocument/publishDiagnostics, got {}",
                notification.method
            )));
        }

        let params = notification
            .params
            .as_ref()
            .ok_or_else(|| LspError::InvalidMessage("missing params".to_string()))?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidMessage("missing uri in params".to_string()))?;

        let diagnostics_val = params
            .get("diagnostics")
            .ok_or_else(|| LspError::InvalidMessage("missing diagnostics array in params".to_string()))?;

        let diagnostics: Vec<Diagnostic> = serde_json::from_value(diagnostics_val.clone())?;
        let path = uri_to_path(uri);

        self.publish_diagnostics(path.clone(), diagnostics);
        Ok(path)
    }

    /// Returns all diagnostics associated with a file path.
    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics
            .read()
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns diagnostics spanning a specific line number (0-based).
    pub fn diagnostics_for_line(&self, path: &Path, line: u32) -> Vec<Diagnostic> {
        self.get_diagnostics(path)
            .into_iter()
            .filter(|d| d.range.start.line <= line && d.range.end.line >= line)
            .collect()
    }

    /// Clears diagnostics for a specific file.
    pub fn clear_diagnostics(&self, path: &Path) {
        self.diagnostics.write().remove(path);
    }

    /// Clears diagnostics across all tracked files.
    pub fn clear_all_diagnostics(&self) {
        self.diagnostics.write().clear();
    }

    /// Returns error count for a given file.
    pub fn error_count(&self, path: &Path) -> usize {
        self.get_diagnostics(path)
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::Error))
            .count()
    }

    /// Returns warning count for a given file.
    pub fn warning_count(&self, path: &Path) -> usize {
        self.get_diagnostics(path)
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::Warning))
            .count()
    }

    // -----------------------------------------------------------------------
    // Autocompletion & Hover Queries
    // -----------------------------------------------------------------------

    /// Creates an LSP `textDocument/completion` request and registers it.
    pub fn request_completion(&self, path: &Path, position: Position, prefix: &str) -> RpcRequest {
        let uri = path_to_uri(path);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });

        let req = self.create_request("textDocument/completion", Some(params));
        self.active_completion_prefix.write().insert(req.id.clone(), prefix.to_string());
        req
    }

    /// Parses an LSP completion response (supporting either `CompletionList` or raw `Vec<CompletionItem>`).
    pub fn parse_completion_response(&self, response: &RpcResponse) -> Result<Vec<CompletionItem>, LspError> {
        if let Some(err) = &response.error {
            return Err(LspError::Rpc {
                code: err.code,
                message: err.message.clone(),
            });
        }

        let result = match &response.result {
            Some(res) if !res.is_null() => res,
            _ => return Ok(Vec::new()),
        };

        if let Ok(items) = serde_json::from_value::<Vec<CompletionItem>>(result.clone()) {
            return Ok(items);
        }

        if let Some(items_val) = result.get("items") {
            let items: Vec<CompletionItem> = serde_json::from_value(items_val.clone())?;
            return Ok(items);
        }

        Ok(Vec::new())
    }

    /// Creates an LSP `textDocument/hover` request.
    pub fn request_hover(&self, path: &Path, position: Position) -> RpcRequest {
        let uri = path_to_uri(path);
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });

        self.create_request("textDocument/hover", Some(params))
    }

    /// Parses an LSP hover response into a `HoverInfo`.
    pub fn parse_hover_response(&self, response: &RpcResponse) -> Result<Option<HoverInfo>, LspError> {
        if let Some(err) = &response.error {
            return Err(LspError::Rpc {
                code: err.code,
                message: err.message.clone(),
            });
        }

        let result = match &response.result {
            Some(res) if !res.is_null() => res,
            _ => return Ok(None),
        };

        let range = result
            .get("range")
            .and_then(|r| serde_json::from_value::<Range>(r.clone()).ok());

        let contents = match result.get("contents") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Object(map)) => {
                // MarkupContent { kind, value }
                map.get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            }
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Object(m) => m
                        .get("value")
                        .and_then(|val| val.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
            _ => return Ok(None),
        };

        if contents.is_empty() {
            Ok(None)
        } else {
            Ok(Some(HoverInfo { contents, range }))
        }
    }
}
