//! Release benchmark for the index engine on a real large file.
//! Usage: cargo run --release --example bench -- /path/to/big.json [search-term]

use huge_json_viewer_lib::export::{export_csv, export_xml};
use huge_json_viewer_lib::index::{build_index, run_search, Doc, Index};
use huge_json_viewer_lib::model::{CsvOptions, XmlOptions};
use memmap2::Mmap;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("usage: bench <file> [term]");
    let term = std::env::args().nth(2).unwrap_or_else(|| "Alice".to_string());

    let file = std::fs::File::open(&path).expect("open");
    let size = file.metadata().unwrap().len();
    let mmap = unsafe { Mmap::map(&file) }.expect("mmap");

    let t = Instant::now();
    let progress = AtomicU64::new(0);
    let (nodes, ndjson) = build_index(&mmap, &progress).expect("index");
    let index_ms = t.elapsed().as_millis();
    let node_count = nodes.len();

    println!("file          : {} ({:.2} MB)", path, size as f64 / 1_048_576.0);
    println!("indexed in    : {} ms  ({:.0} MB/s)", index_ms, size as f64 / 1_048_576.0 / (index_ms as f64 / 1000.0));
    println!("nodes         : {}", node_count);
    println!("bytes/node    : {:.1}", size as f64 / node_count as f64);
    println!("ndjson        : {}", ndjson);

    let file_len = mmap.len() as u64;
    let idx = Arc::new(Index { mmap, nodes, file_len, ndjson });

    let t = Instant::now();
    let mut doc = Doc::new(path.clone(), "bench".into(), idx.clone(), 0);
    println!("visible rows  : {} (built in {} ms)", doc.visible_count(), t.elapsed().as_millis());

    // Fetch a window from the middle.
    let mid = (doc.visible_count() / 2) as usize;
    let t = Instant::now();
    let rows = doc.rows(mid, 60);
    println!("get_rows(mid,60): {} rows in {} µs", rows.len(), t.elapsed().as_micros());

    // Expand the first container child, if any.
    if doc.visible_count() > 1 {
        let t = Instant::now();
        let (added, _, _) = doc.toggle(1);
        println!("toggle row 1  : +{} rows in {} µs", added, t.elapsed().as_micros());
    }

    // Search.
    let t = Instant::now();
    let (matches, capped) = run_search(&idx, &term, true, true, false, false, 200_000).unwrap();
    println!(
        "search '{}'   : {} matches{} in {} ms",
        term,
        matches.len(),
        if capped { "+" } else { "" },
        t.elapsed().as_millis()
    );

    // Reveal the last found match.
    if let Some(m) = matches.last() {
        let t = Instant::now();
        if let Some(vi) = doc.reveal(m.node) {
            println!("reveal match  : row {} in {} ms", vi, t.elapsed().as_millis());
        }
    }

    // CSV export of the whole document.
    let csv_path = format!("{}.export.csv", path);
    let t = Instant::now();
    {
        let f = std::fs::File::create(&csv_path).unwrap();
        let mut w = BufWriter::with_capacity(1 << 20, f);
        let stats = export_csv(
            &idx,
            0,
            &mut w,
            &CsvOptions::default(),
            &AtomicU64::new(0),
            &AtomicBool::new(false),
        )
        .unwrap();
        w.flush().unwrap();
        let sz = std::fs::metadata(&csv_path).unwrap().len();
        println!(
            "csv export    : {} rows, {} cols → {:.1} MB in {} ms",
            stats.rows,
            stats.columns,
            sz as f64 / 1_048_576.0,
            t.elapsed().as_millis()
        );
    }
    std::fs::remove_file(&csv_path).ok();

    // XML export of the whole document.
    let xml_path = format!("{}.export.xml", path);
    let t = Instant::now();
    {
        let f = std::fs::File::create(&xml_path).unwrap();
        let mut w = BufWriter::with_capacity(1 << 20, f);
        export_xml(
            &idx,
            0,
            &mut w,
            &XmlOptions { pretty: false, ..Default::default() },
            &AtomicU64::new(0),
            &AtomicBool::new(false),
        )
        .unwrap();
        w.flush().unwrap();
        let sz = std::fs::metadata(&xml_path).unwrap().len();
        println!("xml export    : {:.1} MB in {} ms", sz as f64 / 1_048_576.0, t.elapsed().as_millis());
    }
    std::fs::remove_file(&xml_path).ok();
}
