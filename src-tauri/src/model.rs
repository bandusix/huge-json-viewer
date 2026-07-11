//! Serde types shared with the frontend over Tauri IPC.

use serde::{Deserialize, Serialize};

/// Summary returned after a file is opened and indexed.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenSummary {
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub node_count: u64,
    pub visible_count: u64,
    pub root_kind: String,
    pub load_ms: u64,
    pub ndjson: bool,
    /// Present when several files were unioned into one view.
    pub union: Option<UnionInfo>,
}

/// Info about a multi-file union open.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnionInfo {
    pub file_count: u32,
    pub skipped: Vec<SkippedFileInfo>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkippedFileInfo {
    pub name: String,
    pub error: String,
}

// ---- Export (JSON -> CSV / XML) ------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CsvOptions {
    /// "comma" | "semicolon" | "tab"
    pub delimiter: String,
    pub bom: bool,
    pub crlf: bool,
    pub null_as_empty: bool,
    pub sanitize_formulas: bool,
    pub max_columns: usize,
    /// Per-cell byte cap (0 = unlimited).
    pub cell_cap: usize,
    pub nested_as_json: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: "comma".into(),
            bom: true,
            crlf: true,
            null_as_empty: true,
            sanitize_formulas: true,
            max_columns: 4096,
            cell_cap: 32767,
            nested_as_json: true,
        }
    }
}

impl CsvOptions {
    pub fn delimiter_byte(&self) -> u8 {
        match self.delimiter.as_str() {
            "semicolon" => b';',
            "tab" => b'\t',
            _ => b',',
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct XmlOptions {
    pub pretty: bool,
    pub declaration: bool,
    pub root_name: String,
    pub item_name: String,
    pub cell_cap: usize,
    pub preserve_keys_attr: bool,
}

impl Default for XmlOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            declaration: true,
            root_name: "root".into(),
            item_name: "item".into(),
            cell_cap: 1_048_576,
            preserve_keys_attr: true,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub node_id: u32,
    /// "csv" | "xml"
    pub format: String,
    pub dest: String,
    #[serde(default)]
    pub csv: CsvOptions,
    #[serde(default)]
    pub xml: XmlOptions,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportStats {
    pub rows: u64,
    pub columns: u32,
    pub bytes_written: u64,
    pub cells_truncated: u64,
    pub canceled: bool,
}

/// Streaming export progress (emitted on the `export-progress` channel).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub rows: u64,
}

/// A single rendered tree row (only visible rows are ever materialized).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RowView {
    /// Node id (index into the flat node arrays).
    pub id: u32,
    /// 1-based line number in the current visible order.
    pub line: u64,
    pub depth: u16,
    /// "object" | "array" | "string" | "number" | "bool" | "null"
    pub kind: &'static str,
    /// Object member key (decoded), or null for array elements / root.
    pub key: Option<String>,
    pub key_truncated: bool,
    pub container: bool,
    pub expanded: bool,
    pub child_count: u32,
    /// Scalar value preview (decoded, truncated). None for containers.
    pub preview: Option<String>,
    pub preview_truncated: bool,
}

/// A window of visible rows plus the current total (for scrollbar sizing).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RowsResponse {
    pub rows: Vec<RowView>,
    pub visible_count: u64,
}

/// Streaming indexing progress (emitted on the `index-progress` channel).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub nodes: u64,
}

/// Result of expanding/collapsing a container.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToggleResult {
    pub added: u64,
    pub removed: u64,
    pub visible_count: u64,
    /// New expanded state of the toggled node.
    pub expanded: bool,
}

/// One breadcrumb segment on the path from root to a node.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PathSeg {
    /// "key" (object member) or "index" (array element) or "root".
    pub kind: &'static str,
    pub label: String,
}

/// Result of a search scan.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub total: u64,
    /// True when the match list hit the storage cap (there may be more).
    pub capped: bool,
    pub query_ms: u64,
}

/// Text extracted from a node for the clipboard (key / value / raw JSON / path).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeText {
    pub text: String,
    /// True when the value was larger than the clipboard cap and got truncated.
    pub truncated: bool,
    /// Full byte length of the underlying value (before any cap).
    pub byte_len: u64,
}

/// Result of revealing (scrolling to) a match or node.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RevealResult {
    pub node_id: u32,
    pub visible_index: u64,
    pub visible_count: u64,
    pub is_key: bool,
}
