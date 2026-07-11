pub mod export;
pub mod index;
pub mod model;

use index::{build_index, kind_str, run_search, Doc, Index};
use model::*;

use parking_lot::Mutex;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

/// Cap on stored match locations (bounds memory; UI shows "N+").
const SEARCH_CAP: usize = 200_000;
/// Uniquifies scratch temp-file names for multi-file union.
static UNION_CTR: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct AppState {
    doc: Mutex<Option<Doc>>,
    export_cancel: Arc<AtomicBool>,
}

fn fmt_perr(bytes: &[u8], pe: index::ParseError) -> String {
    let (line, col) = index::line_col(bytes, pe.offset);
    format!("{} — line {}, column {} (byte {})", pe.msg, line, col, pe.offset)
}

/// Create a self-cleaning scratch file (used for the union buffer and pasted
/// text). On Unix we unlink it immediately — the open fd (and the later mmap)
/// keep the inode alive, so every error path auto-cleans with no leak. Windows
/// cannot delete an open file, so we open it with FILE_FLAG_DELETE_ON_CLOSE +
/// share-delete: the OS removes it once the handle and the memory-mapped view
/// are both closed (deletion is deferred while the view is live).
fn create_scratch(path: &Path) -> std::io::Result<std::fs::File> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_SHARE_DELETE: u32 = 0x0000_0004;
        const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
        std::fs::File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(path)
    }
    #[cfg(not(windows))]
    {
        let f = std::fs::File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let _ = std::fs::remove_file(path);
        Ok(f)
    }
}

/// Open + memory-map + index a file on a background thread, emitting
/// `index-progress` events while the streaming tokenizer runs.
#[tauri::command]
async fn open_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<OpenSummary, String> {
    let meta = std::fs::metadata(&path).map_err(|e| format!("Cannot read file: {e}"))?;
    let size = meta.len();
    if size == 0 {
        return Err("The file is empty.".into());
    }
    if size >= u32::MAX as u64 {
        return Err("Files up to 4 GB are supported in this version.".into());
    }

    let progress = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));

    // Progress watcher: emit throttled progress until indexing finishes.
    {
        let p = progress.clone();
        let d = done.clone();
        let app2 = app.clone();
        std::thread::spawn(move || loop {
            let bytes_done = p.load(Ordering::Relaxed);
            let _ = app2.emit(
                "index-progress",
                ProgressEvent { bytes_done, bytes_total: size, nodes: 0 },
            );
            if d.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(90));
        });
    }

    let path_for_job = path.clone();
    let progress_job = progress.clone();
    let job = tauri::async_runtime::spawn_blocking(move || -> Result<(Index, u64), String> {
        let t0 = Instant::now();
        let file =
            std::fs::File::open(&path_for_job).map_err(|e| format!("Cannot open file: {e}"))?;
        let mmap =
            unsafe { memmap2::Mmap::map(&file) }.map_err(|e| format!("Cannot memory-map file: {e}"))?;
        // madvise hints are a Unix-only optimization (no-op elsewhere); memmap2
        // only exposes `advise`/`Advice` on Unix, so gate them to keep Windows building.
        #[cfg(unix)]
        let _ = mmap.advise(memmap2::Advice::Sequential);
        let (nodes, ndjson) = build_index(&mmap, &progress_job).map_err(|pe| fmt_perr(&mmap, pe))?;
        #[cfg(unix)]
        let _ = mmap.advise(memmap2::Advice::Random);
        let file_len = mmap.len() as u64;
        let idx = Index { mmap, nodes, file_len, ndjson };
        Ok((idx, t0.elapsed().as_millis() as u64))
    })
    .await;

    done.store(true, Ordering::Relaxed);
    let (index_data, load_ms) = job.map_err(|e| format!("Indexing task failed: {e}"))??;

    let index = Arc::new(index_data);
    let file_name = Path::new(&path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    let root_kind = kind_str(index.nodes.kind[0]).to_string();
    let ndjson = index.ndjson;

    let doc = Doc::new(path.clone(), file_name.clone(), index, load_ms);
    let summary = OpenSummary {
        path,
        file_name,
        file_size: size,
        node_count: doc.node_count(),
        visible_count: doc.visible_count(),
        root_kind,
        load_ms,
        ndjson,
        union: None,
    };
    *state.doc.lock() = Some(doc);
    Ok(summary)
}

#[tauri::command]
fn close_file(state: State<'_, AppState>) {
    *state.doc.lock() = None;
}

#[tauri::command]
fn get_rows(state: State<'_, AppState>, start: u64, count: u32) -> Result<RowsResponse, String> {
    let g = state.doc.lock();
    let doc = g.as_ref().ok_or("No file is open.")?;
    Ok(RowsResponse {
        rows: doc.rows(start as usize, count as usize),
        visible_count: doc.visible_count(),
    })
}

#[tauri::command]
fn toggle(state: State<'_, AppState>, vis_index: u64) -> Result<ToggleResult, String> {
    let mut g = state.doc.lock();
    let doc = g.as_mut().ok_or("No file is open.")?;
    let (added, removed, expanded) = doc.toggle(vis_index as usize);
    Ok(ToggleResult {
        added,
        removed,
        visible_count: doc.visible_count(),
        expanded,
    })
}

#[tauri::command]
fn collapse_all(state: State<'_, AppState>) -> Result<u64, String> {
    let mut g = state.doc.lock();
    let doc = g.as_mut().ok_or("No file is open.")?;
    doc.collapse_all();
    Ok(doc.visible_count())
}

#[tauri::command]
fn breadcrumb(state: State<'_, AppState>, node_id: u32) -> Result<Vec<PathSeg>, String> {
    let g = state.doc.lock();
    let doc = g.as_ref().ok_or("No file is open.")?;
    Ok(doc.path_of(node_id))
}

/// Extract clipboard text from a node: `what` ∈ "key" | "value" | "json" | "path".
#[tauri::command]
fn node_text(state: State<'_, AppState>, node_id: u32, what: String) -> Result<NodeText, String> {
    let g = state.doc.lock();
    let doc = g.as_ref().ok_or("No file is open.")?;
    doc.node_text(node_id, &what).ok_or_else(|| "Invalid node or field.".into())
}

#[tauri::command]
fn reveal_node(state: State<'_, AppState>, node_id: u32) -> Result<RevealResult, String> {
    let mut g = state.doc.lock();
    let doc = g.as_mut().ok_or("No file is open.")?;
    let vi = doc.reveal(node_id).ok_or("Node is not reachable.")?;
    Ok(RevealResult {
        node_id,
        visible_index: vi,
        visible_count: doc.visible_count(),
        is_key: false,
    })
}

#[tauri::command]
fn reveal_match(state: State<'_, AppState>, index: u64) -> Result<RevealResult, String> {
    let mut g = state.doc.lock();
    let doc = g.as_mut().ok_or("No file is open.")?;
    let m = *doc.matches.get(index as usize).ok_or("Match out of range.")?;
    let vi = doc.reveal(m.node).ok_or("Cannot reveal match.")?;
    Ok(RevealResult {
        node_id: m.node,
        visible_index: vi,
        visible_count: doc.visible_count(),
        is_key: m.is_key,
    })
}

#[tauri::command]
async fn search(
    state: State<'_, AppState>,
    query: String,
    keys: bool,
    values: bool,
    case_sensitive: bool,
    regex: bool,
) -> Result<SearchResult, String> {
    let index = {
        let g = state.doc.lock();
        g.as_ref().ok_or("No file is open.")?.index.clone()
    };
    let t0 = Instant::now();
    let q = query;
    let res = tauri::async_runtime::spawn_blocking(move || {
        run_search(&index, &q, keys, values, case_sensitive, regex, SEARCH_CAP)
    })
    .await
    .map_err(|e| format!("Search task failed: {e}"))?;
    let (matches, capped) = res?;
    let total = matches.len() as u64;
    let query_ms = t0.elapsed().as_millis() as u64;
    {
        let mut g = state.doc.lock();
        if let Some(doc) = g.as_mut() {
            doc.matches = matches;
        }
    }
    Ok(SearchResult { total, capped, query_ms })
}

#[tauri::command]
fn cancel_export(state: State<'_, AppState>) {
    state.export_cancel.store(true, Ordering::Relaxed);
}

/// Stream a node's subtree to a CSV or XML file on disk.
#[tauri::command]
async fn export(
    app: AppHandle,
    state: State<'_, AppState>,
    req: ExportRequest,
) -> Result<ExportStats, String> {
    let index = {
        let g = state.doc.lock();
        g.as_ref().ok_or("No file is open.")?.index.clone()
    };
    let node_id = req.node_id;
    if node_id as usize >= index.nodes.len() {
        return Err("Invalid node.".into());
    }

    // Byte span of the exported subtree → progress denominator.
    let start_byte = index.nodes.val_start[node_id as usize] as u64;
    let end_node = index.nodes.subtree_end[node_id as usize] as usize;
    let end_byte = if end_node < index.nodes.len() {
        index.nodes.val_start[end_node] as u64
    } else {
        index.file_len
    };
    let total = end_byte.saturating_sub(start_byte).max(1);

    state.export_cancel.store(false, Ordering::Relaxed);
    let cancel = state.export_cancel.clone();

    let progress = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    {
        let p = progress.clone();
        let d = done.clone();
        let app2 = app.clone();
        std::thread::spawn(move || loop {
            let bytes_done = p.load(Ordering::Relaxed);
            let _ = app2.emit(
                "export-progress",
                ExportProgress { bytes_done, bytes_total: total, rows: 0 },
            );
            if d.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(90));
        });
    }

    let progress_job = progress.clone();
    let job = tauri::async_runtime::spawn_blocking(move || -> Result<ExportStats, String> {
        let dest = req.dest.clone();
        let file =
            std::fs::File::create(&dest).map_err(|e| format!("Cannot create file: {e}"))?;
        let mut w = BufWriter::with_capacity(1 << 20, file);
        let result = match req.format.as_str() {
            "json" => export::export_json(&index, node_id, &mut w, &progress_job, &cancel),
            "xml" => export::export_xml(&index, node_id, &mut w, &req.xml, &progress_job, &cancel),
            _ => export::export_csv(&index, node_id, &mut w, &req.csv, &progress_job, &cancel),
        };
        match result {
            Ok(mut stats) => {
                if let Err(e) = w.flush() {
                    drop(w);
                    let _ = std::fs::remove_file(&dest);
                    return Err(e.to_string());
                }
                let f = match w.into_inner() {
                    Ok(f) => f,
                    Err(e) => {
                        let _ = std::fs::remove_file(&dest);
                        return Err(e.to_string());
                    }
                };
                stats.bytes_written = f.metadata().map(|m| m.len()).unwrap_or(0);
                drop(f);
                if stats.canceled {
                    let _ = std::fs::remove_file(&dest);
                }
                Ok(stats)
            }
            Err(e) => {
                drop(w);
                let _ = std::fs::remove_file(&dest);
                Err(e)
            }
        }
    })
    .await;

    done.store(true, Ordering::Relaxed);
    job.map_err(|e| format!("Export task failed: {e}"))?
}

/// Index JSON text pasted from the clipboard. The text is written to a
/// self-cleaning temp file so it can be memory-mapped like any other document.
#[tauri::command]
async fn open_text(state: State<'_, AppState>, text: String) -> Result<OpenSummary, String> {
    if text.trim().is_empty() {
        return Err("There is no JSON text on the clipboard.".into());
    }
    let size = text.len() as u64;
    if size >= u32::MAX as u64 {
        return Err("Pasted text up to 4 GB is supported.".into());
    }

    let job = tauri::async_runtime::spawn_blocking(move || -> Result<(Index, u64), String> {
        let t0 = Instant::now();
        let ctr = UNION_CTR.fetch_add(1, Ordering::Relaxed);
        let scratch_path =
            std::env::temp_dir().join(format!("hjv_paste_{}_{}.json", std::process::id(), ctr));
        let mut scratch =
            create_scratch(&scratch_path).map_err(|e| format!("Cannot create temp file: {e}"))?;
        scratch
            .write_all(text.as_bytes())
            .map_err(|e| format!("Cannot write temp file: {e}"))?;
        scratch.flush().map_err(|e| e.to_string())?;
        let mmap = unsafe { memmap2::Mmap::map(&scratch) }
            .map_err(|e| format!("Cannot memory-map text: {e}"))?;
        drop(scratch);
        let progress = AtomicU64::new(0);
        let (nodes, ndjson) = build_index(&mmap, &progress).map_err(|pe| fmt_perr(&mmap, pe))?;
        let file_len = mmap.len() as u64;
        Ok((Index { mmap, nodes, file_len, ndjson }, t0.elapsed().as_millis() as u64))
    })
    .await;

    let (index_data, load_ms) = job.map_err(|e| format!("Indexing task failed: {e}"))??;
    let index = Arc::new(index_data);
    let root_kind = kind_str(index.nodes.kind[0]).to_string();
    let ndjson = index.ndjson;
    let doc = Doc::new("(pasted)".into(), "Pasted JSON".into(), index, load_ms);
    let summary = OpenSummary {
        path: "(pasted)".into(),
        file_name: "Pasted JSON".into(),
        file_size: size,
        node_count: doc.node_count(),
        visible_count: doc.visible_count(),
        root_kind,
        load_ms,
        ndjson,
        union: None,
    };
    *state.doc.lock() = Some(doc);
    Ok(summary)
}

/// Open several JSON files unioned into one tree (one file → normal open).
#[tauri::command]
async fn open_union(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<OpenSummary, String> {
    if paths.is_empty() {
        return Err("No files selected.".into());
    }
    if paths.len() == 1 {
        return open_file(app, state, paths.into_iter().next().unwrap()).await;
    }

    let total_size: u64 = paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();
    if total_size >= u32::MAX as u64 {
        return Err("The combined files exceed 4 GB (the multi-file union limit).".into());
    }

    let progress = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    {
        let p = progress.clone();
        let d = done.clone();
        let app2 = app.clone();
        std::thread::spawn(move || loop {
            let bytes_done = p.load(Ordering::Relaxed);
            let _ = app2.emit(
                "index-progress",
                ProgressEvent { bytes_done, bytes_total: total_size, nodes: 0 },
            );
            if d.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(90));
        });
    }

    let t0 = Instant::now();
    let paths_job = paths.clone();
    let progress_job = progress.clone();
    let job = tauri::async_runtime::spawn_blocking(
        move || -> Result<(index::Index, u32, Vec<SkippedFileInfo>), String> {
            let ctr = UNION_CTR.fetch_add(1, Ordering::Relaxed);
            let scratch_path = std::env::temp_dir()
                .join(format!("hjv_union_{}_{}.bin", std::process::id(), ctr));
            let mut scratch =
                create_scratch(&scratch_path).map_err(|e| format!("Cannot create scratch file: {e}"))?;
            let ub = index::build_union(&paths_job, &mut scratch, &progress_job)?;
            scratch.flush().map_err(|e| e.to_string())?;
            let index::UnionBuild { nodes, byte_len, files, skipped } = ub;
            let mmap = unsafe { memmap2::Mmap::map(&scratch) }
                .map_err(|e| format!("Cannot memory-map union: {e}"))?;
            drop(scratch);
            let index = index::Index { mmap, nodes, file_len: byte_len, ndjson: false };
            let skipped_info = skipped
                .into_iter()
                .map(|s| SkippedFileInfo { name: s.name, error: s.error })
                .collect();
            Ok((index, files.len() as u32, skipped_info))
        },
    )
    .await;

    done.store(true, Ordering::Relaxed);
    let (index_data, file_count, skipped) = job.map_err(|e| format!("Union task failed: {e}"))??;

    let index = Arc::new(index_data);
    let load_ms = t0.elapsed().as_millis() as u64;
    let root_kind = kind_str(index.nodes.kind[0]).to_string();
    let doc = Doc::new(
        "(union)".into(),
        format!("Union of {file_count} files"),
        index,
        load_ms,
    );
    let summary = OpenSummary {
        path: "(union)".into(),
        file_name: format!("Union of {file_count} files"),
        file_size: total_size,
        node_count: doc.node_count(),
        visible_count: doc.visible_count(),
        root_kind,
        load_ms,
        ndjson: false,
        union: Some(UnionInfo { file_count, skipped }),
    };
    *state.doc.lock() = Some(doc);
    Ok(summary)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            open_file,
            close_file,
            get_rows,
            toggle,
            collapse_all,
            breadcrumb,
            node_text,
            reveal_node,
            reveal_match,
            search,
            export,
            cancel_export,
            open_text,
            open_union
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
