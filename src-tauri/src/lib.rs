pub mod index;
pub mod model;

use index::{build_index, kind_str, run_search, Doc, Index};
use model::*;

use parking_lot::Mutex;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

/// Cap on stored match locations (bounds memory; UI shows "N+").
const SEARCH_CAP: usize = 200_000;

#[derive(Default)]
struct AppState {
    doc: Mutex<Option<Doc>>,
}

fn fmt_perr(bytes: &[u8], pe: index::ParseError) -> String {
    let (line, col) = index::line_col(bytes, pe.offset);
    format!("{} — line {}, column {} (byte {})", pe.msg, line, col, pe.offset)
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
        let _ = mmap.advise(memmap2::Advice::Sequential);
        let (nodes, ndjson) = build_index(&mmap, &progress_job).map_err(|pe| fmt_perr(&mmap, pe))?;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            open_file,
            close_file,
            get_rows,
            toggle,
            collapse_all,
            breadcrumb,
            reveal_node,
            reveal_match,
            search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
