//! Streaming JSON → CSV / XML export, driven by the flat index.
//!
//! Everything reads the mmap and writes a caller-supplied BufWriter. Heap is
//! O(columns) + one row for CSV and O(depth) for XML — independent of file size.

use crate::index::{
    decode_string, is_container, scan_number_end, value_end, Index, K_BOOL, K_NULL, K_NUMBER,
    K_OBJECT, K_STRING, NONE,
};
use crate::model::{CsvOptions, ExportStats, XmlOptions};

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[inline]
fn w_all<W: Write>(w: &mut W, b: &[u8]) -> Result<(), String> {
    w.write_all(b).map_err(|e| e.to_string())
}

fn cap_of(cell_cap: usize) -> usize {
    if cell_cap == 0 {
        1 << 20
    } else {
        cell_cap
    }
}

// ============================ JSON ========================================

/// Stream a node's subtree verbatim as JSON. Since the index is built from the
/// original file, the value's raw bytes already ARE valid JSON, so this just
/// brace-matches the value's extent and copies those bytes — O(1) heap, any size.
pub fn export_json<W: Write>(
    idx: &Index,
    root: u32,
    w: &mut W,
    progress: &AtomicU64,
    cancel: &AtomicBool,
) -> Result<ExportStats, String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let start = nodes.val_start[root as usize] as usize;
    let kind = nodes.kind[root as usize];
    let (end, _) = value_end(bytes, start, kind, bytes.len());
    let mut stats = ExportStats::default();
    let mut i = start;
    let chunk = 1 << 20;
    while i < end {
        if cancel.load(Ordering::Relaxed) {
            stats.canceled = true;
            return Ok(stats);
        }
        let j = (i + chunk).min(end);
        w_all(w, &bytes[i..j])?;
        stats.bytes_written += (j - i) as u64;
        progress.store(stats.bytes_written, Ordering::Relaxed);
        i = j;
    }
    Ok(stats)
}

// ============================ CSV =========================================

struct Columns {
    names: Vec<String>,
    index: HashMap<Vec<u8>, u32>,
    has_key: bool,
    has_value: bool,
}

fn discover_columns(idx: &Index, root: u32, cap: usize) -> Result<Columns, String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let mut names: Vec<String> = Vec::new();
    let mut index: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut has_key = false;
    let mut has_value = false;

    let end = nodes.subtree_end[root as usize];
    let mut c = root + 1;
    while c < end {
        if nodes.key_start[c as usize] != NONE {
            has_key = true;
        }
        if nodes.kind[c as usize] == K_OBJECT {
            let cend = nodes.subtree_end[c as usize];
            let mut m = c + 1;
            while m < cend {
                let ks = nodes.key_start[m as usize];
                if ks != NONE {
                    let (name, _) = decode_string(bytes, ks, 512);
                    let kb = name.clone().into_bytes();
                    if !index.contains_key(&kb) {
                        if names.len() >= cap {
                            return Err(format!(
                                "This node has more than {} distinct columns — too many for one CSV. \
                                 Select a narrower subtree (e.g. an inner array), or export as XML.",
                                cap
                            ));
                        }
                        index.insert(kb, names.len() as u32);
                        names.push(name);
                    }
                }
                m = nodes.subtree_end[m as usize];
            }
        } else {
            has_value = true;
        }
        c = nodes.subtree_end[c as usize];
    }
    Ok(Columns { names, index, has_key, has_value })
}

fn write_quoted<W: Write>(w: &mut W, raw: &[u8]) -> Result<(), String> {
    w_all(w, b"\"")?;
    let mut last = 0;
    for (i, &b) in raw.iter().enumerate() {
        if b == b'"' {
            w_all(w, &raw[last..i])?;
            w_all(w, b"\"\"")?;
            last = i + 1;
        }
    }
    w_all(w, &raw[last..])?;
    w_all(w, b"\"")
}

/// Decode a JSON string body (starting at `content_start`, after the opening
/// quote), then write it as a quoted, RFC-4180-escaped CSV field.
fn write_string_field<W: Write>(
    w: &mut W,
    bytes: &[u8],
    content_start: u32,
    cell_cap: usize,
    opt: &CsvOptions,
    stats: &mut ExportStats,
) -> Result<(), String> {
    let (mut s, trunc) = decode_string(bytes, content_start, cell_cap);
    if trunc {
        stats.cells_truncated += 1;
        s.push('\u{2026}');
    }
    w_all(w, b"\"")?;
    if opt.sanitize_formulas {
        if let Some(&f) = s.as_bytes().first() {
            if matches!(f, b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r') {
                w_all(w, b"'")?;
            }
        }
    }
    let sb = s.as_bytes();
    let mut last = 0;
    for (i, &b) in sb.iter().enumerate() {
        if b == b'"' {
            w_all(w, &sb[last..i])?;
            w_all(w, b"\"\"")?;
            last = i + 1;
        }
    }
    w_all(w, &sb[last..])?;
    w_all(w, b"\"")
}

/// Stream a nested object/array as a raw-JSON CSV cell (quoted, `"`→`""`).
fn write_json_field<W: Write>(
    w: &mut W,
    bytes: &[u8],
    val_start: u32,
    cell_cap: usize,
    stats: &mut ExportStats,
) -> Result<(), String> {
    w_all(w, b"\"")?;
    let n = bytes.len();
    let mut i = val_start as usize;
    let mut depth: i64 = 0;
    let mut in_str = false;
    let mut esc = false;
    let mut written = 0usize;
    let mut truncated = false;
    while i < n {
        // Only truncate on a UTF-8 char boundary (not mid multi-byte sequence).
        if written >= cell_cap && (bytes[i] & 0xC0) != 0x80 {
            truncated = true;
            break;
        }
        let b = bytes[i];
        if b == b'"' {
            w_all(w, b"\"\"")?;
            written += 2;
        } else {
            w_all(w, &bytes[i..i + 1])?;
            written += 1;
        }
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
        } else {
            match b {
                b'"' => in_str = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => {
                    depth -= 1;
                    if depth <= 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    if truncated {
        stats.cells_truncated += 1;
        w_all(w, "\u{2026}".as_bytes())?;
    }
    w_all(w, b"\"")
}

fn write_value_cell<W: Write>(
    idx: &Index,
    node: u32,
    w: &mut W,
    opt: &CsvOptions,
    cell_cap: usize,
    stats: &mut ExportStats,
) -> Result<(), String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let vs = nodes.val_start[node as usize] as usize;
    match nodes.kind[node as usize] {
        K_NUMBER => {
            // Numbers are emitted in full — width is semantically meaningful and
            // a number never needs CSV quoting.
            let e = scan_number_end(bytes, vs);
            w_all(w, &bytes[vs..e])
        }
        K_BOOL => w_all(w, if bytes.get(vs) == Some(&b't') { b"true" } else { b"false" }),
        K_NULL => {
            if opt.null_as_empty {
                Ok(())
            } else {
                w_all(w, b"null")
            }
        }
        K_STRING => write_string_field(w, bytes, (vs + 1) as u32, cell_cap, opt, stats),
        _ => {
            if opt.nested_as_json {
                write_json_field(w, bytes, vs as u32, cell_cap, stats)
            } else {
                Ok(())
            }
        }
    }
}

pub fn export_csv<W: Write>(
    idx: &Index,
    root: u32,
    w: &mut W,
    opt: &CsvOptions,
    progress: &AtomicU64,
    cancel: &AtomicBool,
) -> Result<ExportStats, String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let delim = opt.delimiter_byte();
    let nl: &[u8] = if opt.crlf { b"\r\n" } else { b"\n" };
    let cell_cap = cap_of(opt.cell_cap);
    let mut stats = ExportStats::default();
    let start_byte = nodes.val_start[root as usize] as u64;

    if opt.bom {
        w_all(w, &[0xEF, 0xBB, 0xBF])?;
    }

    // Scalar root → single "value" row.
    if !is_container(nodes.kind[root as usize]) {
        w_all(w, b"value")?;
        w_all(w, nl)?;
        write_value_cell(idx, root, w, opt, cell_cap, &mut stats)?;
        w_all(w, nl)?;
        stats.rows = 1;
        stats.columns = 1;
        return Ok(stats);
    }

    let cols = discover_columns(idx, root, opt.max_columns)?;
    let name_base = if cols.has_key { 1 } else { 0 };
    let ncols = name_base + cols.names.len() + if cols.has_value { 1 } else { 0 };
    let value_col = if cols.has_value { Some(ncols - 1) } else { None };
    stats.columns = ncols as u32;

    // Header.
    let mut first = true;
    let sep = |w: &mut W, first: &mut bool| -> Result<(), String> {
        if !*first {
            w_all(w, &[delim])?;
        }
        *first = false;
        Ok(())
    };
    if cols.has_key {
        sep(w, &mut first)?;
        write_quoted(w, b"_key")?;
    }
    for name in &cols.names {
        sep(w, &mut first)?;
        write_quoted(w, name.as_bytes())?;
    }
    if cols.has_value {
        sep(w, &mut first)?;
        write_quoted(w, b"_value")?;
    }
    w_all(w, nl)?;

    // Rows.
    let mut row: Vec<u32> = vec![NONE; ncols];
    let end = nodes.subtree_end[root as usize];
    let mut c = root + 1;
    let mut counter = 0u64;
    while c < end {
        for x in row.iter_mut() {
            *x = NONE;
        }
        if nodes.kind[c as usize] == K_OBJECT {
            let cend = nodes.subtree_end[c as usize];
            let mut m = c + 1;
            while m < cend {
                let ks = nodes.key_start[m as usize];
                if ks != NONE {
                    let (name, _) = decode_string(bytes, ks, 512);
                    if let Some(&col) = cols.index.get(name.as_bytes()) {
                        row[name_base + col as usize] = m; // last wins
                    }
                }
                m = nodes.subtree_end[m as usize];
            }
        } else if let Some(vc) = value_col {
            row[vc] = c;
        }

        let mut first = true;
        for (ci, &slot) in row.iter().enumerate() {
            sep(w, &mut first)?;
            if cols.has_key && ci == 0 {
                let ks = nodes.key_start[c as usize];
                if ks != NONE {
                    write_string_field(w, bytes, ks, cell_cap, opt, &mut stats)?;
                }
            } else if slot != NONE {
                write_value_cell(idx, slot, w, opt, cell_cap, &mut stats)?;
            }
        }
        w_all(w, nl)?;
        stats.rows += 1;

        c = nodes.subtree_end[c as usize];
        counter += 1;
        if counter & 0xFFF == 0 {
            let pos = nodes.val_start[c.min(end - 1) as usize] as u64;
            progress.store(pos.saturating_sub(start_byte), Ordering::Relaxed);
            if cancel.load(Ordering::Relaxed) {
                stats.canceled = true;
                return Ok(stats);
            }
        }
    }
    Ok(stats)
}

// ============================ XML =========================================

/// XML 1.0 NameStartChar (excluding ':' to avoid namespace confusion).
fn is_xml_name_start(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '_')
        || matches!(
            c as u32,
            0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2FF | 0x370..=0x37D | 0x37F..=0x1FFF
                | 0x200C..=0x200D | 0x2070..=0x218F | 0x2C00..=0x2FEF | 0x3001..=0xD7FF
                | 0xF900..=0xFDCF | 0xFDF0..=0xFFFD | 0x10000..=0xEFFFF
        )
}

/// XML 1.0 NameChar.
fn is_xml_name_char(c: char) -> bool {
    is_xml_name_start(c)
        || matches!(c, '-' | '.' | '0'..='9')
        || matches!(c as u32, 0xB7 | 0x300..=0x36F | 0x203F..=0x2040)
}

/// Sanitize a JSON key into a valid XML element name. Returns (name, changed).
pub fn sanitize_xml_name(key: &str) -> (String, bool) {
    if key.is_empty() {
        return ("_".into(), true);
    }
    let mut out = String::with_capacity(key.len());
    let mut changed = false;
    for ch in key.chars() {
        if is_xml_name_char(ch) {
            out.push(ch);
        } else {
            out.push('_');
            changed = true;
        }
    }
    // First char must additionally be a NameStartChar (e.g. a leading digit is
    // a valid NameChar but not a NameStartChar → prefix '_').
    let first = out.chars().next().unwrap();
    if !is_xml_name_start(first) {
        out.insert(0, '_');
        changed = true;
    }
    (out, changed)
}

fn xml_escape_text<W: Write>(w: &mut W, s: &str) -> Result<(), String> {
    let mut buf = [0u8; 4];
    for ch in s.chars() {
        match ch {
            '&' => w_all(w, b"&amp;")?,
            '<' => w_all(w, b"&lt;")?,
            '>' => w_all(w, b"&gt;")?,
            c if !xml_char_ok(c) => w_all(w, "\u{FFFD}".as_bytes())?,
            c => w_all(w, c.encode_utf8(&mut buf).as_bytes())?,
        }
    }
    Ok(())
}

/// Valid XML 1.0 Char? (`\t \n \r`, 0x20..0xD7FF, 0xE000..0xFFFD, 0x10000..0x10FFFF).
fn xml_char_ok(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r')
        || matches!(c as u32, 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}

fn xml_escape_attr<W: Write>(w: &mut W, s: &str) -> Result<(), String> {
    let mut buf = [0u8; 4];
    for ch in s.chars() {
        match ch {
            '&' => w_all(w, b"&amp;")?,
            '<' => w_all(w, b"&lt;")?,
            '>' => w_all(w, b"&gt;")?,
            '"' => w_all(w, b"&quot;")?,
            c if !xml_char_ok(c) => w_all(w, "\u{FFFD}".as_bytes())?,
            c => w_all(w, c.encode_utf8(&mut buf).as_bytes())?,
        }
    }
    Ok(())
}

fn indent<W: Write>(w: &mut W, depth: usize) -> Result<(), String> {
    let n = (depth * 2).min(200);
    for _ in 0..n {
        w_all(w, b" ")?;
    }
    Ok(())
}

fn write_name_attr<W: Write>(w: &mut W, name: &str, attr: &Option<String>) -> Result<(), String> {
    w_all(w, name.as_bytes())?;
    if let Some(a) = attr {
        w_all(w, b" key=\"")?;
        xml_escape_attr(w, a)?;
        w_all(w, b"\"")?;
    }
    Ok(())
}

fn child_name(idx: &Index, parent_kind: u8, child: u32, opt: &XmlOptions) -> (String, Option<String>) {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    if parent_kind == K_OBJECT {
        let ks = nodes.key_start[child as usize];
        if ks != NONE {
            let (k, _) = decode_string(bytes, ks, 512);
            let (nm, changed) = sanitize_xml_name(&k);
            let attr = if changed && opt.preserve_keys_attr {
                Some(k)
            } else {
                None
            };
            return (nm, attr);
        }
    }
    (opt.item_name.clone(), None)
}

fn write_scalar_element<W: Write>(
    idx: &Index,
    w: &mut W,
    node: u32,
    name: &str,
    attr: &Option<String>,
    _opt: &XmlOptions,
    cell_cap: usize,
    stats: &mut ExportStats,
) -> Result<(), String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let vs = nodes.val_start[node as usize] as usize;
    let kind = nodes.kind[node as usize];

    if kind == K_NULL {
        w_all(w, b"<")?;
        write_name_attr(w, name, attr)?;
        return w_all(w, b" nil=\"true\"/>");
    }

    w_all(w, b"<")?;
    write_name_attr(w, name, attr)?;
    w_all(w, b">")?;
    match kind {
        K_NUMBER => {
            let e = scan_number_end(bytes, vs);
            w_all(w, &bytes[vs..e])?;
        }
        K_BOOL => w_all(w, if bytes.get(vs) == Some(&b't') { b"true" } else { b"false" })?,
        K_STRING => {
            let (s, trunc) = decode_string(bytes, (vs + 1) as u32, cell_cap);
            if trunc {
                stats.cells_truncated += 1;
            }
            xml_escape_text(w, &s)?;
            if trunc {
                w_all(w, "\u{2026}".as_bytes())?;
            }
        }
        _ => {}
    }
    w_all(w, b"</")?;
    w_all(w, name.as_bytes())?;
    w_all(w, b">")
}

struct XmlFrame {
    end: u32,
    cursor: u32,
    node: u32,
    name: String,
    depth: usize,
    had_child: bool,
}

pub fn export_xml<W: Write>(
    idx: &Index,
    root: u32,
    w: &mut W,
    opt: &XmlOptions,
    progress: &AtomicU64,
    cancel: &AtomicBool,
) -> Result<ExportStats, String> {
    let nodes = &idx.nodes;
    let bytes = idx.bytes();
    let cell_cap = cap_of(opt.cell_cap);
    let mut stats = ExportStats::default();
    let start_byte = nodes.val_start[root as usize] as u64;

    if opt.declaration {
        w_all(w, b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        if opt.pretty {
            w_all(w, b"\n")?;
        }
    }

    // Root element name/attr.
    let (root_name, root_attr) = {
        let ks = nodes.key_start[root as usize];
        if ks != NONE {
            let (k, _) = decode_string(bytes, ks, 512);
            let (nm, changed) = sanitize_xml_name(&k);
            (nm, if changed && opt.preserve_keys_attr { Some(k) } else { None })
        } else {
            (opt.root_name.clone(), None)
        }
    };

    let mut stack: Vec<XmlFrame> = Vec::new();
    emit_node(idx, w, opt, cell_cap, &mut stats, &mut stack, root, root_name, root_attr, 0)?;

    let mut counter = 0u64;
    while !stack.is_empty() {
        let ti = stack.len() - 1;
        if stack[ti].cursor >= stack[ti].end {
            let f = stack.pop().unwrap();
            if opt.pretty && f.had_child {
                indent(w, f.depth)?;
            }
            w_all(w, b"</")?;
            w_all(w, f.name.as_bytes())?;
            w_all(w, b">")?;
            if opt.pretty {
                w_all(w, b"\n")?;
            }
            continue;
        }
        let child = stack[ti].cursor;
        stack[ti].cursor = nodes.subtree_end[child as usize];
        stack[ti].had_child = true;
        let pkind = nodes.kind[stack[ti].node as usize];
        let cdepth = stack[ti].depth + 1;
        let (cname, cattr) = child_name(idx, pkind, child, opt);
        emit_node(idx, w, opt, cell_cap, &mut stats, &mut stack, child, cname, cattr, cdepth)?;

        counter += 1;
        if counter & 0xFFF == 0 {
            let pos = nodes.val_start[child as usize] as u64;
            progress.store(pos.saturating_sub(start_byte), Ordering::Relaxed);
            if cancel.load(Ordering::Relaxed) {
                stats.canceled = true;
                return Ok(stats);
            }
        }
    }
    Ok(stats)
}

#[allow(clippy::too_many_arguments)]
fn emit_node<W: Write>(
    idx: &Index,
    w: &mut W,
    opt: &XmlOptions,
    cell_cap: usize,
    stats: &mut ExportStats,
    stack: &mut Vec<XmlFrame>,
    node: u32,
    name: String,
    attr: Option<String>,
    depth: usize,
) -> Result<(), String> {
    let nodes = &idx.nodes;
    let kind = nodes.kind[node as usize];
    if opt.pretty {
        indent(w, depth)?;
    }
    if is_container(kind) {
        w_all(w, b"<")?;
        write_name_attr(w, &name, &attr)?;
        w_all(w, b">")?;
        if nodes.child_count[node as usize] == 0 {
            w_all(w, b"</")?;
            w_all(w, name.as_bytes())?;
            w_all(w, b">")?;
            if opt.pretty {
                w_all(w, b"\n")?;
            }
        } else {
            if opt.pretty {
                w_all(w, b"\n")?;
            }
            stack.push(XmlFrame {
                end: nodes.subtree_end[node as usize],
                cursor: node + 1,
                node,
                name,
                depth,
                had_child: false,
            });
        }
    } else {
        write_scalar_element(idx, w, node, &name, &attr, opt, cell_cap, stats)?;
        if opt.pretty {
            w_all(w, b"\n")?;
        }
    }
    Ok(())
}

// ============================ tests =======================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{build_index, Index, Nodes};
    use memmap2::Mmap;
    use std::sync::atomic::AtomicUsize;

    static CTR: AtomicUsize = AtomicUsize::new(0);

    fn index_of(s: &str) -> Index {
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("hjv_exp_{}_{}.json", std::process::id(), n));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(s.as_bytes()).unwrap();
        }
        let file = std::fs::File::open(&path).unwrap();
        let mmap = unsafe { Mmap::map(&file) }.unwrap();
        let progress = AtomicU64::new(0);
        let (nodes, ndjson): (Nodes, bool) = build_index(&mmap, &progress).unwrap();
        let file_len = mmap.len() as u64;
        let _ = std::fs::remove_file(&path);
        Index { mmap, nodes, file_len, ndjson }
    }

    fn csv(s: &str, opt: CsvOptions) -> String {
        let idx = index_of(s);
        let mut out: Vec<u8> = Vec::new();
        export_csv(&idx, 0, &mut out, &opt, &AtomicU64::new(0), &AtomicBool::new(false)).unwrap();
        String::from_utf8(out).unwrap()
    }
    fn xml(s: &str, opt: XmlOptions) -> String {
        let idx = index_of(s);
        let mut out: Vec<u8> = Vec::new();
        export_xml(&idx, 0, &mut out, &opt, &AtomicU64::new(0), &AtomicBool::new(false)).unwrap();
        String::from_utf8(out).unwrap()
    }

    fn no_bom() -> CsvOptions {
        CsvOptions { bom: false, crlf: false, ..Default::default() }
    }

    #[test]
    fn csv_array_of_objects() {
        let out = csv(r#"[{"a":1,"b":"x"},{"b":"y","c":true},{"a":2}]"#, no_bom());
        // union of keys in first-seen order: a,b,c
        assert_eq!(
            out,
            "\"a\",\"b\",\"c\"\n1,\"x\",\n,\"y\",true\n2,,\n"
        );
    }

    #[test]
    fn csv_escaping_and_formula_guard() {
        let out = csv(r#"[{"v":"a,b"},{"v":"he\"llo"},{"v":"=SUM(1)"},{"v":"line\nbreak"}]"#, no_bom());
        assert_eq!(
            out,
            "\"v\"\n\"a,b\"\n\"he\"\"llo\"\n\"'=SUM(1)\"\n\"line\nbreak\"\n"
        );
    }

    #[test]
    fn csv_number_fidelity_and_null() {
        let out = csv(r#"[{"n":-3.14e10,"z":null},{"n":123456789012345678901234567890}]"#, no_bom());
        assert_eq!(out, "\"n\",\"z\"\n-3.14e10,\n123456789012345678901234567890,\n");
    }

    #[test]
    fn csv_object_root_and_scalars() {
        let out = csv(r#"{"row1":{"a":1},"row2":{"a":2,"b":3}}"#, no_bom());
        assert_eq!(out, "\"_key\",\"a\",\"b\"\n\"row1\",1,\n\"row2\",2,3\n");
        let out2 = csv(r#"[1,2,3]"#, no_bom());
        assert_eq!(out2, "\"_value\"\n1\n2\n3\n");
        let out3 = csv("42", no_bom());
        assert_eq!(out3, "value\n42\n");
    }

    #[test]
    fn csv_nested_as_json_cell() {
        let out = csv(r#"[{"o":{"x":1},"a":[1,2]}]"#, no_bom());
        assert_eq!(out, "\"o\",\"a\"\n\"{\"\"x\"\":1}\",\"[1,2]\"\n");
    }

    #[test]
    fn csv_bom_and_crlf() {
        let out = csv(r#"[{"a":1}]"#, CsvOptions { bom: true, crlf: true, ..Default::default() });
        assert_eq!(out.as_bytes()[0..3], [0xEF, 0xBB, 0xBF]);
        assert!(out.contains("\r\n"));
    }

    #[test]
    fn csv_column_cap_aborts() {
        let idx = index_of(r#"[{"a":1},{"b":2},{"c":3}]"#);
        let mut out: Vec<u8> = Vec::new();
        let opt = CsvOptions { max_columns: 2, ..Default::default() };
        let r = export_csv(&idx, 0, &mut out, &opt, &AtomicU64::new(0), &AtomicBool::new(false));
        assert!(r.is_err());
    }

    #[test]
    fn xml_basic_and_escaping() {
        let opt = XmlOptions { pretty: false, declaration: false, ..Default::default() };
        let out = xml(r#"{"a":1,"b":"x<y&z","c":[true,null]}"#, opt);
        assert_eq!(
            out,
            "<root><a>1</a><b>x&lt;y&amp;z</b><c><item>true</item><item nil=\"true\"/></c></root>"
        );
    }

    #[test]
    fn xml_name_sanitization() {
        let (n, c) = sanitize_xml_name("a b");
        assert_eq!((n.as_str(), c), ("a_b", true));
        assert_eq!(sanitize_xml_name("3d").0, "_3d");
        assert_eq!(sanitize_xml_name("").0, "_");
        assert_eq!(sanitize_xml_name("ok_name-1.2").0, "ok_name-1.2");
        // invalid-name key becomes an element with a key="..." attribute
        let opt = XmlOptions { pretty: false, declaration: false, ..Default::default() };
        let out = xml(r#"{"a b":1}"#, opt);
        assert_eq!(out, "<root><a_b key=\"a b\">1</a_b></root>");
    }

    #[test]
    fn xml_name_rejects_non_namechar_symbols() {
        // U+00D7 (×) and U+2192 (→) are >= 0x80 but NOT valid XML NameChars.
        assert_eq!(sanitize_xml_name("a×b"), ("a_b".to_string(), true));
        assert_eq!(sanitize_xml_name("a→b"), ("a_b".to_string(), true));
        // A real Unicode letter (é) is a valid NameChar and survives.
        assert_eq!(sanitize_xml_name("café").1, false);
        let opt = XmlOptions { pretty: false, declaration: false, ..Default::default() };
        assert_eq!(xml(r#"{"a×b":1}"#, opt), "<root><a_b key=\"a×b\">1</a_b></root>");
    }

    #[test]
    fn csv_number_emitted_in_full_despite_cap() {
        let out = csv(
            r#"[{"n":123456789}]"#,
            CsvOptions { bom: false, crlf: false, cell_cap: 3, ..Default::default() },
        );
        assert_eq!(out, "\"n\"\n123456789\n");
    }

    #[test]
    fn csv_json_cell_never_splits_utf8() {
        // A cell_cap landing mid multi-byte char must not split it (else the
        // helper's String::from_utf8 would panic on invalid bytes).
        let out = csv(
            r#"[{"o":["aa中"]}]"#,
            CsvOptions { bom: false, crlf: false, cell_cap: 6, ..Default::default() },
        );
        assert!(out.starts_with("\"o\"\n"));
        assert!(out.contains('中'));
    }

    #[test]
    fn xml_replaces_noncharacters() {
        let opt = XmlOptions { pretty: false, declaration: false, ..Default::default() };
        let out = xml("{\"a\":\"x\u{FFFF}y\"}", opt);
        assert_eq!(out, "<root><a>x\u{FFFD}y</a></root>");
    }

    #[test]
    fn xml_deep_no_stack_overflow() {
        // 3000-deep nesting must not blow the native stack (iterative walk).
        let mut s = String::new();
        for _ in 0..3000 {
            s.push_str("[");
        }
        for _ in 0..3000 {
            s.push_str("]");
        }
        let opt = XmlOptions { pretty: false, declaration: false, ..Default::default() };
        let out = xml(&s, opt);
        assert!(out.starts_with("<root>"));
        assert!(out.ends_with("</root>"));
    }
}
