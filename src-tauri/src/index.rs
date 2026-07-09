//! Streaming index + virtualized-tree engine for very large JSON files.
//!
//! The file is memory-mapped and scanned exactly once to build a compact,
//! flat (structure-of-arrays) index. Only visible rows are ever materialized,
//! so browsing a multi-gigabyte file stays instant.
//!
//! Node record (23 bytes/node):
//!   kind:u8  depth:u16  key_start:u32  val_start:u32  parent:u32  child_count:u32  subtree_end:u32
//!
//! Byte offsets are u32 (files up to 4 GB). Node ids are u32.
//! `val_start` is strictly increasing in pre-order, so the visible list is
//! always sorted ascending by id, enabling O(log n) reveal.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use memmap2::Mmap;

// ---- Node kinds -----------------------------------------------------------

pub const K_OBJECT: u8 = 0;
pub const K_ARRAY: u8 = 1;
pub const K_STRING: u8 = 2;
pub const K_NUMBER: u8 = 3;
pub const K_BOOL: u8 = 4;
pub const K_NULL: u8 = 5;

pub const NONE: u32 = u32::MAX;

const MAX_DEPTH: usize = 5000;
/// Guard against pathological node explosion. A realistic 2–3 GB file of records
/// runs ~13 source-bytes/node → up to ~250M nodes for 3 GB, so the cap must clear
/// that. At ~23 bytes/node this bounds the in-heap index at ~6.9 GB worst case
/// (pathological input); typical 2–3 GB files use far less.
const MAX_NODES: usize = 300_000_000;
/// Update the shared progress counter every this-many nodes during indexing.
const PROGRESS_STRIDE: usize = 1 << 20;

const KEY_PREVIEW_BYTES: usize = 300;
const VAL_PREVIEW_BYTES: usize = 400;
const NUM_PREVIEW_BYTES: usize = 120;

#[inline]
pub fn is_container(kind: u8) -> bool {
    kind == K_OBJECT || kind == K_ARRAY
}

pub fn kind_str(kind: u8) -> &'static str {
    match kind {
        K_OBJECT => "object",
        K_ARRAY => "array",
        K_STRING => "string",
        K_NUMBER => "number",
        K_BOOL => "bool",
        _ => "null",
    }
}

// ---- Parse error ----------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ParseError {
    pub offset: usize,
    pub msg: &'static str,
}

#[inline]
fn perr(offset: usize, msg: &'static str) -> ParseError {
    ParseError { offset, msg }
}

/// 1-based (line, column) for an error message.
pub fn line_col(bytes: &[u8], offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    let end = offset.min(bytes.len());
    for &b in &bytes[..end] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

// ---- Flat node arrays -----------------------------------------------------

#[derive(Default)]
pub struct Nodes {
    pub kind: Vec<u8>,
    pub depth: Vec<u16>,
    pub key_start: Vec<u32>, // content start (after opening quote), or NONE
    pub val_start: Vec<u32>, // start byte of the value token
    pub parent: Vec<u32>,    // NONE for root
    pub child_count: Vec<u32>,
    pub subtree_end: Vec<u32>, // index after the whole subtree (exclusive)
}

impl Nodes {
    fn with_capacity(byte_len: usize) -> Self {
        let est = (byte_len / 24).clamp(1024, MAX_NODES);
        Nodes {
            kind: Vec::with_capacity(est),
            depth: Vec::with_capacity(est),
            key_start: Vec::with_capacity(est),
            val_start: Vec::with_capacity(est),
            parent: Vec::with_capacity(est),
            child_count: Vec::with_capacity(est),
            subtree_end: Vec::with_capacity(est),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.kind.len()
    }

    #[inline]
    fn push(&mut self, kind: u8, depth: u16, key_start: u32, val_start: u32, parent: u32) -> u32 {
        let id = self.kind.len() as u32;
        self.kind.push(kind);
        self.depth.push(depth);
        self.key_start.push(key_start);
        self.val_start.push(val_start);
        self.parent.push(parent);
        self.child_count.push(0);
        self.subtree_end.push(0);
        if parent != NONE {
            self.child_count[parent as usize] += 1;
        }
        id
    }
}

// ---- Low-level scanners ---------------------------------------------------

#[inline]
fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r'
}

#[inline]
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    while i < n && is_ws(bytes[i]) {
        i += 1;
    }
    i
}

/// `pos` at the opening quote; returns position just after the closing quote.
pub(crate) fn scan_string_end(bytes: &[u8], pos: usize) -> Result<usize, ParseError> {
    let n = bytes.len();
    let mut i = pos + 1;
    while i < n {
        match bytes[i] {
            b'\\' => i += 2, // skip escaped char (also covers \uXXXX)
            b'"' => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err(perr(pos, "unterminated string"))
}

/// `pos` at first byte of a number; returns first byte after the number span.
pub(crate) fn scan_number_end(bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    while i < n {
        match bytes[i] {
            b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E' => i += 1,
            _ => break,
        }
    }
    i
}

#[inline]
fn is_value_start(b: u8) -> bool {
    matches!(b, b'{' | b'[' | b'"' | b't' | b'f' | b'n' | b'-' | b'0'..=b'9')
}

// ---- Tokenizer / index builder -------------------------------------------

struct Frame {
    node: u32,
    is_object: bool,
    need_comma: bool,
}

fn open_value(
    bytes: &[u8],
    pos: usize,
    nodes: &mut Nodes,
    stack: &mut Vec<Frame>,
    parent: u32,
    depth: u16,
    key_start: u32,
) -> Result<usize, ParseError> {
    if nodes.len() >= MAX_NODES {
        return Err(perr(pos, "file exceeds the maximum number of JSON nodes for this version"));
    }
    let n = bytes.len();
    match bytes[pos] {
        b'{' => {
            let id = nodes.push(K_OBJECT, depth, key_start, pos as u32, parent);
            stack.push(Frame { node: id, is_object: true, need_comma: false });
            Ok(pos + 1)
        }
        b'[' => {
            let id = nodes.push(K_ARRAY, depth, key_start, pos as u32, parent);
            stack.push(Frame { node: id, is_object: false, need_comma: false });
            Ok(pos + 1)
        }
        b'"' => {
            let id = nodes.push(K_STRING, depth, key_start, pos as u32, parent);
            nodes.subtree_end[id as usize] = id + 1;
            scan_string_end(bytes, pos)
        }
        b't' => {
            if pos + 4 <= n && &bytes[pos..pos + 4] == b"true" {
                let id = nodes.push(K_BOOL, depth, key_start, pos as u32, parent);
                nodes.subtree_end[id as usize] = id + 1;
                Ok(pos + 4)
            } else {
                Err(perr(pos, "invalid literal; expected 'true'"))
            }
        }
        b'f' => {
            if pos + 5 <= n && &bytes[pos..pos + 5] == b"false" {
                let id = nodes.push(K_BOOL, depth, key_start, pos as u32, parent);
                nodes.subtree_end[id as usize] = id + 1;
                Ok(pos + 5)
            } else {
                Err(perr(pos, "invalid literal; expected 'false'"))
            }
        }
        b'n' => {
            if pos + 4 <= n && &bytes[pos..pos + 4] == b"null" {
                let id = nodes.push(K_NULL, depth, key_start, pos as u32, parent);
                nodes.subtree_end[id as usize] = id + 1;
                Ok(pos + 4)
            } else {
                Err(perr(pos, "invalid literal; expected 'null'"))
            }
        }
        b'-' | b'0'..=b'9' => {
            let id = nodes.push(K_NUMBER, depth, key_start, pos as u32, parent);
            nodes.subtree_end[id as usize] = id + 1;
            Ok(scan_number_end(bytes, pos))
        }
        _ => Err(perr(pos, "unexpected character; expected a JSON value")),
    }
}

fn step(bytes: &[u8], pos: usize, nodes: &mut Nodes, stack: &mut Vec<Frame>) -> Result<usize, ParseError> {
    let top = stack.len() - 1;
    let is_object = stack[top].is_object;
    let cnode = stack[top].node;
    let n = bytes.len();

    let mut p = skip_ws(bytes, pos);
    if p >= n {
        return Err(perr(pos, if is_object { "unterminated object" } else { "unterminated array" }));
    }

    let close = if is_object { b'}' } else { b']' };
    if bytes[p] == close {
        nodes.subtree_end[cnode as usize] = nodes.len() as u32;
        stack.pop();
        return Ok(p + 1);
    }

    if stack[top].need_comma {
        if bytes[p] == b',' {
            p = skip_ws(bytes, p + 1);
            if p >= n {
                return Err(perr(pos, "unterminated container"));
            }
            stack[top].need_comma = false;
        } else {
            return Err(perr(p, if is_object { "expected ',' or '}'" } else { "expected ',' or ']'" }));
        }
    }

    let cdepth = nodes.depth[cnode as usize] + 1;
    if cdepth as usize > MAX_DEPTH {
        return Err(perr(p, "maximum nesting depth exceeded"));
    }

    if is_object {
        if bytes[p] != b'"' {
            return Err(perr(p, "expected string key"));
        }
        let key_content = (p + 1) as u32;
        let after_key = scan_string_end(bytes, p)?;
        let mut q = skip_ws(bytes, after_key);
        if q >= n || bytes[q] != b':' {
            return Err(perr(q.min(n.saturating_sub(1)), "expected ':' after key"));
        }
        q = skip_ws(bytes, q + 1);
        if q >= n {
            return Err(perr(q.min(n.saturating_sub(1)), "expected a value after ':'"));
        }
        let np = open_value(bytes, q, nodes, stack, cnode, cdepth, key_content)?;
        stack[top].need_comma = true;
        Ok(np)
    } else {
        let np = open_value(bytes, p, nodes, stack, cnode, cdepth, NONE)?;
        stack[top].need_comma = true;
        Ok(np)
    }
}

fn parse_value_tree(
    bytes: &[u8],
    pos: usize,
    nodes: &mut Nodes,
    parent: u32,
    depth: u16,
    key_start: u32,
    progress: &AtomicU64,
) -> Result<usize, ParseError> {
    let mut stack: Vec<Frame> = Vec::new();
    let mut p = open_value(bytes, pos, nodes, &mut stack, parent, depth, key_start)?;
    while !stack.is_empty() {
        p = step(bytes, p, nodes, &mut stack)?;
        if nodes.len() & (PROGRESS_STRIDE - 1) == 0 {
            progress.store(p as u64, Ordering::Relaxed);
        }
    }
    Ok(p)
}

fn build_ndjson(bytes: &[u8], start: usize, progress: &AtomicU64) -> Result<Nodes, ParseError> {
    let mut nodes = Nodes::with_capacity(bytes.len());
    let root = nodes.push(K_ARRAY, 0, NONE, start as u32, NONE);
    let n = bytes.len();
    let mut pos = skip_ws(bytes, start);
    while pos < n {
        let mut stack: Vec<Frame> = Vec::new();
        let mut p = open_value(bytes, pos, &mut nodes, &mut stack, root, 1, NONE)?;
        while !stack.is_empty() {
            p = step(bytes, p, &mut nodes, &mut stack)?;
            if nodes.len() & (PROGRESS_STRIDE - 1) == 0 {
                progress.store(p as u64, Ordering::Relaxed);
            }
        }
        pos = skip_ws(bytes, p);
    }
    nodes.subtree_end[root as usize] = nodes.len() as u32;
    Ok(nodes)
}

/// Build the index for a whole buffer. Returns (nodes, is_ndjson).
/// `progress` is updated with the current byte offset during the scan.
pub fn build_index(bytes: &[u8], progress: &AtomicU64) -> Result<(Nodes, bool), ParseError> {
    let start = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) { 3 } else { 0 };
    let first = skip_ws(bytes, start);
    if first >= bytes.len() {
        return Err(perr(first, "the file is empty"));
    }

    let mut nodes = Nodes::with_capacity(bytes.len());
    let after = parse_value_tree(bytes, first, &mut nodes, NONE, 0, NONE, progress)?;
    let trailing = skip_ws(bytes, after);
    if trailing >= bytes.len() {
        return Ok((nodes, false));
    }
    // Trailing content: treat as NDJSON / concatenated JSON if it starts a value.
    if is_value_start(bytes[trailing]) {
        let nodes2 = build_ndjson(bytes, start, progress)?;
        return Ok((nodes2, true));
    }
    Err(perr(trailing, "unexpected trailing characters after JSON value"))
}

// ---- Union of multiple files ---------------------------------------------

/// One successfully merged file.
pub struct FileEntry {
    pub name: String,
    pub start: u64, // byte offset of the file's bytes within the scratch buffer
    pub end: u64,
}

/// A file that could not be opened/parsed and was skipped.
pub struct SkippedFile {
    pub name: String,
    pub error: String,
}

pub struct UnionBuild {
    pub nodes: Nodes,
    pub byte_len: u64,
    pub files: Vec<FileEntry>,
    pub skipped: Vec<SkippedFile>,
}

/// Minimal JSON-string escape for a filename used as a synthetic object key.
fn escape_label(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Merge several JSON files into one synthetic root object whose members are the
/// files (keyed by filename). Each file is indexed independently with
/// `build_index` (so NDJSON files stay correct), its raw bytes are copied into
/// `scratch`, and its node arrays are rebased into one merged `Nodes`. The
/// result is a single contiguous byte buffer with strictly-increasing
/// `val_start`, so every downstream feature (tree, search, export) works as-is.
pub fn build_union(
    paths: &[String],
    scratch: &mut File,
    progress: &AtomicU64,
) -> Result<UnionBuild, String> {
    let est_bytes: u64 = paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();
    let mut nodes = Nodes::with_capacity(est_bytes as usize);

    // Synthetic root object at node 0, byte 0 = '{'.
    scratch
        .write_all(b"{")
        .map_err(|e| format!("Cannot write scratch file: {e}"))?;
    let _root = nodes.push(K_OBJECT, 0, NONE, 0, NONE);
    let mut offset: u64 = 1;

    let mut files: Vec<FileEntry> = Vec::new();
    let mut skipped: Vec<SkippedFile> = Vec::new();
    let mut used_labels: HashMap<String, u32> = HashMap::new();

    for path in paths {
        let base_name = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        // Deduplicate display labels (dup filename -> "name (2)").
        let label_name = match used_labels.get_mut(&base_name) {
            Some(n) => {
                *n += 1;
                format!("{} ({})", base_name, n)
            }
            None => {
                used_labels.insert(base_name.clone(), 1);
                base_name.clone()
            }
        };

        let src_file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                skipped.push(SkippedFile { name: label_name, error: format!("cannot open: {e}") });
                continue;
            }
        };
        let src = match unsafe { Mmap::map(&src_file) } {
            Ok(m) => m,
            Err(e) => {
                skipped.push(SkippedFile { name: label_name, error: format!("cannot map: {e}") });
                continue;
            }
        };

        let local_progress = AtomicU64::new(0);
        let (local, _ndjson) = match build_index(&src, &local_progress) {
            Ok(x) => x,
            Err(pe) => {
                let (line, col) = line_col(&src, pe.offset);
                skipped.push(SkippedFile {
                    name: label_name,
                    error: format!("{} (line {}, column {})", pe.msg, line, col),
                });
                continue;
            }
        };

        // Capacity guards (fail before committing this file).
        if nodes.len() + local.len() > MAX_NODES {
            return Err("The combined files have too many JSON nodes for this version.".into());
        }
        let label = escape_label(&label_name);
        let projected = offset + 2 + label.len() as u64 + src.len() as u64;
        if projected >= u32::MAX as u64 {
            return Err("The combined size exceeds 4 GB (the current limit for multi-file union).".into());
        }

        // Write the label:  "name"
        scratch.write_all(b"\"").map_err(|e| e.to_string())?;
        let label_content = offset + 1;
        scratch.write_all(label.as_bytes()).map_err(|e| e.to_string())?;
        scratch.write_all(b"\"").map_err(|e| e.to_string())?;
        offset += 2 + label.len() as u64;

        // Copy the file's raw bytes verbatim.
        let byte_base = offset;
        scratch.write_all(&src).map_err(|e| e.to_string())?;
        offset += src.len() as u64;
        progress.store(offset, Ordering::Relaxed);

        // Rebase the file's nodes into the merged arrays.
        let n_base = nodes.len() as u32;
        let base_off = byte_base as u32;
        let lc = label_content as u32;
        for j in 0..local.len() {
            nodes.kind.push(local.kind[j]);
            nodes.depth.push(local.depth[j] + 1);
            nodes.key_start.push(if j == 0 {
                lc
            } else if local.key_start[j] == NONE {
                NONE
            } else {
                local.key_start[j] + base_off
            });
            nodes.val_start.push(local.val_start[j] + base_off);
            nodes.parent.push(if j == 0 { 0 } else { local.parent[j] + n_base });
            nodes.child_count.push(local.child_count[j]);
            nodes.subtree_end.push(local.subtree_end[j] + n_base);
        }
        nodes.child_count[0] += 1;
        files.push(FileEntry { name: label_name, start: byte_base, end: offset });
    }

    if files.is_empty() {
        return Err("None of the selected files could be opened as JSON.".into());
    }
    nodes.subtree_end[0] = nodes.len() as u32;
    scratch.flush().map_err(|e| e.to_string())?;

    Ok(UnionBuild { nodes, byte_len: offset, files, skipped })
}

// ---- Decoders (for rendering visible rows only) ---------------------------

#[inline]
fn push_char(out: &mut Vec<u8>, ch: char) {
    let mut b = [0u8; 4];
    out.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
}

fn hex4(b: &[u8]) -> Option<u16> {
    if b.len() < 4 {
        return None;
    }
    let mut v: u16 = 0;
    for &d in &b[..4] {
        let n = match d {
            b'0'..=b'9' => d - b'0',
            b'a'..=b'f' => d - b'a' + 10,
            b'A'..=b'F' => d - b'A' + 10,
            _ => return None,
        };
        v = (v << 4) | n as u16;
    }
    Some(v)
}

/// Decode a JSON string body starting at `content_start` (after the opening
/// quote), up to `max_bytes` of decoded output. Returns (decoded, truncated).
pub fn decode_string(bytes: &[u8], content_start: u32, max_bytes: usize) -> (String, bool) {
    let n = bytes.len();
    let mut i = content_start as usize;
    let mut out: Vec<u8> = Vec::new();
    let mut truncated = false;

    while i < n {
        let c = bytes[i];
        if c == b'"' {
            break;
        }
        if out.len() >= max_bytes {
            truncated = true;
            break;
        }
        if c == b'\\' {
            i += 1;
            if i >= n {
                break;
            }
            match bytes[i] {
                b'"' => { out.push(b'"'); i += 1; }
                b'\\' => { out.push(b'\\'); i += 1; }
                b'/' => { out.push(b'/'); i += 1; }
                b'n' => { out.push(b'\n'); i += 1; }
                b't' => { out.push(b'\t'); i += 1; }
                b'r' => { out.push(b'\r'); i += 1; }
                b'b' => { out.push(0x08); i += 1; }
                b'f' => { out.push(0x0C); i += 1; }
                b'u' => {
                    if i + 5 <= n {
                        if let Some(hi) = hex4(&bytes[i + 1..i + 5]) {
                            if (0xD800..=0xDBFF).contains(&hi)
                                && i + 11 <= n
                                && bytes[i + 5] == b'\\'
                                && bytes[i + 6] == b'u'
                            {
                                if let Some(lo) = hex4(&bytes[i + 7..i + 11]) {
                                    if (0xDC00..=0xDFFF).contains(&lo) {
                                        let cp = 0x10000u32
                                            + (((hi as u32) - 0xD800) << 10)
                                            + ((lo as u32) - 0xDC00);
                                        push_char(&mut out, char::from_u32(cp).unwrap_or('\u{FFFD}'));
                                        i += 11;
                                    } else {
                                        push_char(&mut out, '\u{FFFD}');
                                        i += 5;
                                    }
                                } else {
                                    push_char(&mut out, '\u{FFFD}');
                                    i += 5;
                                }
                            } else {
                                push_char(&mut out, char::from_u32(hi as u32).unwrap_or('\u{FFFD}'));
                                i += 5;
                            }
                        } else {
                            out.push(b'u');
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                other => { out.push(other); i += 1; }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), truncated)
}

pub(crate) fn scalar_end(bytes: &[u8], val_start: usize, kind: u8) -> usize {
    match kind {
        K_STRING => scan_string_end(bytes, val_start).unwrap_or(val_start + 1),
        K_NUMBER => scan_number_end(bytes, val_start),
        K_BOOL => {
            if bytes.get(val_start) == Some(&b't') {
                val_start + 4
            } else {
                val_start + 5
            }
        }
        K_NULL => val_start + 4,
        _ => val_start,
    }
}

// ---- Search ---------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Match {
    pub node: u32,
    #[allow(dead_code)] // kept for future exact-span highlighting
    pub offset: u32,
    pub is_key: bool,
}

/// Map a raw byte offset to the (node, is_key) it belongs to, if any.
fn classify(idx: &Index, o: usize) -> Option<(u32, bool)> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();

    // Value hit: deepest node whose value starts at/before `o`.
    let jp = nodes.val_start.partition_point(|&x| (x as usize) <= o);
    if jp > 0 {
        let j = jp - 1;
        let k = nodes.kind[j];
        if k >= K_STRING {
            let s = nodes.val_start[j] as usize;
            if o < scalar_end(bytes, s, k) {
                return Some((j as u32, false));
            }
        }
    }

    // Key hit: the node whose value starts just after `o` (its key precedes it).
    if jp < nodes.len() {
        let k = jp;
        let ks = nodes.key_start[k];
        if ks != NONE {
            let ks_us = ks as usize;
            if ks_us <= o {
                let end = scan_string_end(bytes, ks_us - 1).unwrap_or(ks_us);
                if o < end.saturating_sub(1) {
                    return Some((k as u32, true));
                }
            }
        }
    }
    None
}

pub fn run_search(
    idx: &Index,
    query: &str,
    keys: bool,
    values: bool,
    case_sensitive: bool,
    regex: bool,
    cap: usize,
) -> Result<(Vec<Match>, bool), String> {
    let bytes = idx.bytes();
    let mut out: Vec<Match> = Vec::new();
    let mut capped = false;
    if query.is_empty() || (!keys && !values) {
        return Ok((out, false));
    }

    let accept = |is_key: bool| (is_key && keys) || (!is_key && values);

    if !regex && case_sensitive {
        let finder = memchr::memmem::Finder::new(query.as_bytes());
        let mut base = 0usize;
        while let Some(rel) = finder.find(&bytes[base..]) {
            let o = base + rel;
            if let Some((node, is_key)) = classify(idx, o) {
                if accept(is_key) {
                    out.push(Match { node, offset: o as u32, is_key });
                    if out.len() >= cap {
                        capped = true;
                        break;
                    }
                }
            }
            base = o + 1;
        }
    } else {
        let mut pat = if regex { query.to_string() } else { regex::escape(query) };
        if !case_sensitive {
            pat = format!("(?i){}", pat);
        }
        let re = regex::bytes::RegexBuilder::new(&pat)
            .size_limit(64 << 20)
            .build()
            .map_err(|e| format!("Invalid pattern: {}", e))?;
        for m in re.find_iter(bytes) {
            if m.start() == m.end() {
                continue;
            }
            let o = m.start();
            if let Some((node, is_key)) = classify(idx, o) {
                if accept(is_key) {
                    out.push(Match { node, offset: o as u32, is_key });
                    if out.len() >= cap {
                        capped = true;
                        break;
                    }
                }
            }
        }
    }
    Ok((out, capped))
}

// ---- Immutable index + mutable document -----------------------------------

pub struct Index {
    pub mmap: Mmap,
    pub nodes: Nodes,
    #[allow(dead_code)] // reserved for a future "file info" command
    pub file_len: u64,
    pub ndjson: bool,
}

impl Index {
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.mmap
    }
}

pub(crate) fn direct_children(nodes: &Nodes, id: u32) -> Vec<u32> {
    let end = nodes.subtree_end[id as usize];
    let mut v = Vec::with_capacity(nodes.child_count[id as usize] as usize);
    let mut c = id + 1;
    while c < end {
        v.push(c);
        c = nodes.subtree_end[c as usize];
    }
    v
}

/// Append the visible descendants of `id` (pre-order), honoring `expanded`.
fn collect_expanded(nodes: &Nodes, expanded: &HashSet<u32>, id: u32, out: &mut Vec<u32>) {
    // (cursor, end) frames; iterative to avoid recursion on deep trees.
    let mut stack: Vec<(u32, u32)> = vec![(id + 1, nodes.subtree_end[id as usize])];
    while let Some(frame) = stack.last_mut() {
        let (c, end) = (frame.0, frame.1);
        if c >= end {
            stack.pop();
            continue;
        }
        frame.0 = nodes.subtree_end[c as usize]; // advance to next sibling
        out.push(c);
        if nodes.child_count[c as usize] > 0 && expanded.contains(&c) {
            stack.push((c + 1, nodes.subtree_end[c as usize]));
        }
    }
}

#[allow(dead_code)] // path/file_name/load_ms are retained metadata
pub struct Doc {
    pub path: String,
    pub file_name: String,
    pub index: Arc<Index>,
    pub expanded: HashSet<u32>,
    pub visible: Vec<u32>,
    pub matches: Vec<Match>,
    pub load_ms: u64,
}

impl Doc {
    pub fn new(path: String, file_name: String, index: Arc<Index>, load_ms: u64) -> Self {
        let mut expanded = HashSet::new();
        let mut visible = Vec::new();
        visible.push(0);
        let nodes = &index.nodes;
        if is_container(nodes.kind[0]) && nodes.child_count[0] > 0 {
            expanded.insert(0);
            collect_expanded(nodes, &expanded, 0, &mut visible);
        }
        Doc {
            path,
            file_name,
            index,
            expanded,
            visible,
            matches: Vec::new(),
            load_ms,
        }
    }

    #[inline]
    pub fn visible_count(&self) -> u64 {
        self.visible.len() as u64
    }

    #[inline]
    pub fn node_count(&self) -> u64 {
        self.index.nodes.len() as u64
    }

    #[allow(dead_code)]
    pub fn is_expanded(&self, id: u32) -> bool {
        self.expanded.contains(&id)
    }

    fn rebuild_visible(&mut self) {
        let nodes = &self.index.nodes;
        self.visible.clear();
        self.visible.push(0);
        if self.expanded.contains(&0) {
            collect_expanded(nodes, &self.expanded, 0, &mut self.visible);
        }
    }

    /// Toggle a container at visible row `vis` (expand ↔ collapse).
    /// Returns (added, removed, expanded_state).
    pub fn toggle(&mut self, vis: usize) -> (u64, u64, bool) {
        if vis >= self.visible.len() {
            return (0, 0, false);
        }
        let id = self.visible[vis];
        let nodes = &self.index.nodes;
        if !is_container(nodes.kind[id as usize]) || nodes.child_count[id as usize] == 0 {
            return (0, 0, false);
        }
        if self.expanded.contains(&id) {
            // collapse: drop the contiguous descendant block (deeper rows).
            self.expanded.remove(&id);
            let d = nodes.depth[id as usize];
            let mut q = vis + 1;
            let vislen = self.visible.len();
            while q < vislen && nodes.depth[self.visible[q] as usize] > d {
                q += 1;
            }
            let removed = (q - (vis + 1)) as u64;
            self.visible.drain(vis + 1..q);
            (0, removed, false)
        } else {
            // expand: splice in visible descendants honoring preserved state.
            self.expanded.insert(id);
            let mut block: Vec<u32> = Vec::new();
            collect_expanded(nodes, &self.expanded, id, &mut block);
            let added = block.len() as u64;
            self.visible.splice(vis + 1..vis + 1, block);
            (added, 0, true)
        }
    }

    /// Collapse everything back to the initial (root + direct children) state.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
        self.visible.clear();
        self.visible.push(0);
        let nodes = &self.index.nodes;
        if is_container(nodes.kind[0]) && nodes.child_count[0] > 0 {
            self.expanded.insert(0);
            for c in direct_children(nodes, 0) {
                self.visible.push(c);
            }
        }
    }

    /// Ensure `id` is visible (expanding ancestors) and return its visible row.
    pub fn reveal(&mut self, id: u32) -> Option<u64> {
        let nodes = &self.index.nodes;
        if (id as usize) >= nodes.len() {
            return None;
        }
        // Expand all ancestors.
        let mut p = nodes.parent[id as usize];
        let mut changed = false;
        while p != NONE {
            if self.expanded.insert(p) {
                changed = true;
            }
            p = nodes.parent[p as usize];
        }
        if changed || id != 0 {
            self.rebuild_visible();
        }
        // visible is sorted ascending by id → binary search.
        match self.visible.binary_search(&id) {
            Ok(i) => Some(i as u64),
            Err(_) => None,
        }
    }

    /// Breadcrumb path from root to `id`.
    pub fn path_of(&self, id: u32) -> Vec<crate::model::PathSeg> {
        use crate::model::PathSeg;
        let nodes = &self.index.nodes;
        if (id as usize) >= nodes.len() {
            return Vec::new();
        }
        let bytes = self.index.bytes();
        let mut chain: Vec<u32> = Vec::new();
        let mut cur = id;
        loop {
            chain.push(cur);
            let par = nodes.parent[cur as usize];
            if par == NONE {
                break;
            }
            cur = par;
        }
        chain.reverse();

        let mut segs = Vec::with_capacity(chain.len());
        for (i, &node) in chain.iter().enumerate() {
            if i == 0 {
                segs.push(PathSeg { kind: "root", label: "$".to_string() });
                continue;
            }
            let ks = nodes.key_start[node as usize];
            if ks != NONE {
                let (label, _) = decode_string(bytes, ks, 120);
                segs.push(PathSeg { kind: "key", label });
            } else {
                // Array element: compute ordinal among parent's children (capped).
                let parent = nodes.parent[node as usize];
                let idx = self.child_ordinal(parent, node);
                let label = match idx {
                    Some(n) => n.to_string(),
                    None => "…".to_string(),
                };
                segs.push(PathSeg { kind: "index", label });
            }
        }
        segs
    }

    fn child_ordinal(&self, parent: u32, child: u32) -> Option<u64> {
        let nodes = &self.index.nodes;
        let end = nodes.subtree_end[parent as usize];
        let mut c = parent + 1;
        let mut idx = 0u64;
        let cap = 2_000_000u64;
        while c < end {
            if c == child {
                return Some(idx);
            }
            c = nodes.subtree_end[c as usize];
            idx += 1;
            if idx > cap {
                return None;
            }
        }
        None
    }

    /// Build a RowView for node `id` at visible line `line0` (0-based).
    pub fn row_view(&self, id: u32, line0: usize) -> crate::model::RowView {
        use crate::model::RowView;
        let nodes = &self.index.nodes;
        let bytes = self.index.bytes();
        let kind = nodes.kind[id as usize];
        let container = is_container(kind);

        let (key, key_truncated) = {
            let ks = nodes.key_start[id as usize];
            if ks == NONE {
                (None, false)
            } else {
                let (s, t) = decode_string(bytes, ks, KEY_PREVIEW_BYTES);
                (Some(s), t)
            }
        };

        let (preview, preview_truncated) = if container {
            (None, false)
        } else {
            let vs = nodes.val_start[id as usize];
            match kind {
                K_STRING => {
                    let (s, t) = decode_string(bytes, vs + 1, VAL_PREVIEW_BYTES);
                    (Some(s), t)
                }
                K_NUMBER => {
                    let s = vs as usize;
                    let e = scan_number_end(bytes, s);
                    let capped = (e - s).min(NUM_PREVIEW_BYTES);
                    let trunc = e - s > NUM_PREVIEW_BYTES;
                    (Some(String::from_utf8_lossy(&bytes[s..s + capped]).into_owned()), trunc)
                }
                K_BOOL => {
                    let v = if bytes[vs as usize] == b't' { "true" } else { "false" };
                    (Some(v.to_string()), false)
                }
                _ => (Some("null".to_string()), false),
            }
        };

        RowView {
            id,
            line: line0 as u64 + 1,
            depth: nodes.depth[id as usize],
            kind: kind_str(kind),
            key,
            key_truncated,
            container,
            expanded: self.expanded.contains(&id),
            child_count: nodes.child_count[id as usize],
            preview,
            preview_truncated,
        }
    }

    pub fn rows(&self, start: usize, count: usize) -> Vec<crate::model::RowView> {
        let n = self.visible.len();
        let s = start.min(n);
        let e = (start + count).min(n);
        let mut out = Vec::with_capacity(e - s);
        for vi in s..e {
            let id = self.visible[vi];
            out.push(self.row_view(id, vi));
        }
        out
    }
}

// ---- tests --------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::AtomicUsize;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn index_of(s: &str) -> Arc<Index> {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("hjv_test_{}_{}.json", std::process::id(), n));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(s.as_bytes()).unwrap();
        }
        let file = std::fs::File::open(&path).unwrap();
        let mmap = unsafe { Mmap::map(&file) }.unwrap();
        let progress = AtomicU64::new(0);
        let (nodes, ndjson) = build_index(&mmap, &progress).expect("index build");
        let file_len = mmap.len() as u64;
        let _ = std::fs::remove_file(&path);
        Arc::new(Index { mmap, nodes, file_len, ndjson })
    }

    fn doc_of(s: &str) -> Doc {
        Doc::new("t".into(), "t".into(), index_of(s), 0)
    }

    fn count_values(v: &serde_json::Value) -> u64 {
        match v {
            serde_json::Value::Array(a) => 1 + a.iter().map(count_values).sum::<u64>(),
            serde_json::Value::Object(o) => 1 + o.values().map(count_values).sum::<u64>(),
            _ => 1,
        }
    }

    #[test]
    fn basic_structure_and_toggle() {
        let mut doc = doc_of(r#"{"a":1,"b":[10,20],"c":{"d":true}}"#);
        assert_eq!(doc.node_count(), 7);
        assert_eq!(doc.index.nodes.kind[0], K_OBJECT);
        assert_eq!(doc.index.nodes.child_count[0], 3);
        // root auto-expanded: root + a + b + c
        assert_eq!(doc.visible_count(), 4);

        let rows = doc.rows(0, 100);
        assert_eq!(rows[1].key.as_deref(), Some("a"));
        assert_eq!(rows[1].kind, "number");
        assert_eq!(rows[1].preview.as_deref(), Some("1"));
        assert_eq!(rows[2].key.as_deref(), Some("b"));
        assert!(rows[2].container);
        assert_eq!(rows[2].child_count, 2);

        // expand "b" at visible index 2
        let (added, removed, expanded) = doc.toggle(2);
        assert_eq!((added, removed, expanded), (2, 0, true));
        assert_eq!(doc.visible_count(), 6);
        // collapse it again
        let (added, removed, expanded) = doc.toggle(2);
        assert_eq!((added, removed, expanded), (0, 2, false));
        assert_eq!(doc.visible_count(), 4);
    }

    #[test]
    fn root_scalar() {
        let doc = doc_of("42");
        assert_eq!(doc.node_count(), 1);
        assert_eq!(doc.visible_count(), 1);
        let rows = doc.rows(0, 10);
        assert_eq!(rows[0].kind, "number");
        assert_eq!(rows[0].preview.as_deref(), Some("42"));
        assert!(rows[0].key.is_none());
    }

    #[test]
    fn empty_containers() {
        let doc = doc_of(r#"{"a":{},"b":[]}"#);
        assert_eq!(doc.node_count(), 3);
        let rows = doc.rows(0, 10);
        assert_eq!(rows[1].child_count, 0);
        assert!(rows[1].container);
    }

    #[test]
    fn ndjson_wrap() {
        let doc = doc_of("{\"a\":1}\n{\"b\":2}\n");
        assert!(doc.index.ndjson);
        assert_eq!(doc.index.nodes.kind[0], K_ARRAY);
        assert_eq!(doc.index.nodes.child_count[0], 2);
        // root(1) + obj(1)+a(1) + obj(1)+b(1)
        assert_eq!(doc.node_count(), 5);
    }

    #[test]
    fn escapes_and_surrogates() {
        let doc = doc_of(r#"{"k":"a\nbA😀"}"#);
        let rows = doc.rows(0, 10);
        assert_eq!(rows[1].key.as_deref(), Some("k"));
        assert_eq!(rows[1].preview.as_deref(), Some("a\nbA\u{1F600}"));
    }

    #[test]
    fn search_key_vs_value() {
        let idx = index_of(r#"{"name":"alice","age":30,"tags":["x","name"]}"#);
        // both scopes: key "name" + value "name" in tags
        let (m, _) = run_search(&idx, "name", true, true, true, false, 1000).unwrap();
        assert_eq!(m.len(), 2);
        // keys only
        let (mk, _) = run_search(&idx, "name", true, false, true, false, 1000).unwrap();
        assert_eq!(mk.len(), 1);
        assert!(mk[0].is_key);
        // values only
        let (mv, _) = run_search(&idx, "name", false, true, true, false, 1000).unwrap();
        assert_eq!(mv.len(), 1);
        assert!(!mv[0].is_key);
        // case-insensitive
        let (mi, _) = run_search(&idx, "ALICE", false, true, false, false, 1000).unwrap();
        assert_eq!(mi.len(), 1);
    }

    #[test]
    fn reveal_deep_node() {
        let mut doc = doc_of(r#"{"a":{"b":{"c":[1,2,{"d":"deep"}]}}}"#);
        // node for "d":"deep" — find it by scanning kinds/keys
        let nodes = &doc.index.nodes;
        let bytes = doc.index.bytes();
        let mut target = None;
        for id in 0..nodes.len() as u32 {
            if nodes.key_start[id as usize] != NONE {
                let (k, _) = decode_string(bytes, nodes.key_start[id as usize], 32);
                if k == "d" {
                    target = Some(id);
                }
            }
        }
        let target = target.unwrap();
        let vi = doc.reveal(target).expect("reveal");
        assert_eq!(doc.visible[vi as usize], target);
        let rows = doc.rows(vi as usize, 1);
        assert_eq!(rows[0].preview.as_deref(), Some("deep"));
    }

    #[test]
    fn oracle_node_count() {
        let samples = [
            r#"{"users":[{"id":1,"name":"a","roles":["x","y"]},{"id":2,"name":"b","active":true,"meta":null}],"count":2,"nested":{"deep":{"deeper":[1,2,3,4,5]}}}"#,
            r#"[1,2,3,[4,[5,[6,[7]]]],{"a":{"b":{"c":{}}}},[],{},null,true,false,-3.14e10]"#,
            r#"{"": "empty key", "unicode": "héllo wörld", "arr": []}"#,
        ];
        for s in samples {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            let doc = doc_of(s);
            assert_eq!(doc.node_count(), count_values(&v), "mismatch for {s}");
        }
    }

    #[test]
    fn collapse_all_resets() {
        let mut doc = doc_of(r#"{"a":[1,2],"b":[3,4]}"#);
        doc.toggle(1); // expand a
        assert!(doc.visible_count() > 3);
        doc.collapse_all();
        // root + a + b
        assert_eq!(doc.visible_count(), 3);
    }

    fn write_temp_file(content: &str) -> String {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("hjv_u_{}_{}.json", std::process::id(), n));
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn union_merge_and_invariants() {
        let f1 = write_temp_file(r#"[1,2,3]"#); //          array(1)+3 = 4 nodes
        let f2 = write_temp_file(r#"{"a":1,"b":{"c":2}}"#); // obj+a+b(obj)+c = 4
        let f3 = write_temp_file("{\"x\":1}\n{\"y\":2}\n"); // NDJSON -> array(1)+obj+x+obj+y = 5
        let paths = vec![f1.clone(), f2.clone(), f3.clone()];

        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let scratch_path =
            std::env::temp_dir().join(format!("hjv_scratch_{}_{}.bin", std::process::id(), n));
        let mut scratch = std::fs::File::create(&scratch_path).unwrap();
        let progress = AtomicU64::new(0);
        let ub = build_union(&paths, &mut scratch, &progress).unwrap();
        drop(scratch);

        assert_eq!(ub.nodes.len(), 1 + 4 + 4 + 5);
        assert_eq!(ub.nodes.child_count[0], 3);
        assert!(ub.skipped.is_empty());
        assert_eq!(ub.files.len(), 3);

        // Merge invariants — the fast guard against off-by-one rebasing.
        let n = &ub.nodes;
        for i in 1..n.len() {
            // Non-decreasing (an NDJSON synthetic-array root shares val_start with
            // its first child); that is all partition_point search requires.
            assert!(n.val_start[i] >= n.val_start[i - 1], "val_start decreased at {i}");
            let p = n.parent[i];
            assert!(p == NONE || (p as usize) < i, "parent not before child at {i}");
            assert!(n.subtree_end[i] as usize > i && n.subtree_end[i] as usize <= n.len());
        }

        // Render the merged index; filenames must appear as the member keys.
        let file = std::fs::File::open(&scratch_path).unwrap();
        let mmap = unsafe { Mmap::map(&file) }.unwrap();
        let idx = Arc::new(Index {
            mmap,
            nodes: ub.nodes,
            file_len: ub.byte_len,
            ndjson: false,
        });
        let doc = Doc::new("u".into(), "u".into(), idx.clone(), 0);
        assert_eq!(doc.visible_count(), 4); // root + 3 files
        let rows = doc.rows(0, 10);
        assert_eq!(rows[1].key.as_deref(), Path::new(&f1).file_name().unwrap().to_str());
        assert_eq!(rows[1].kind, "array");
        assert_eq!(rows[1].child_count, 3);

        // Filename search hits the file-root key.
        let fname2 = Path::new(&f2).file_name().unwrap().to_str().unwrap();
        let (mk, _) = run_search(&idx, fname2, true, false, true, false, 100).unwrap();
        assert!(mk.iter().any(|x| x.is_key), "filename search should hit a key");

        // Value "2" appears in all three files.
        let (mv, _) = run_search(&idx, "2", false, true, true, false, 100).unwrap();
        assert!(mv.len() >= 3, "expected >=3 value hits, got {}", mv.len());

        let _ = std::fs::remove_file(&scratch_path);
        for p in [f1, f2, f3] {
            let _ = std::fs::remove_file(p);
        }
    }
}
