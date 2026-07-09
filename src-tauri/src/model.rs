//! Serde types shared with the frontend over Tauri IPC.

use serde::Serialize;

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

/// Result of revealing (scrolling to) a match or node.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RevealResult {
    pub node_id: u32,
    pub visible_index: u64,
    pub visible_count: u64,
    pub is_key: bool,
}
