# Huge JSON Viewer

A fast, native **macOS** desktop app for opening and searching **multi‑gigabyte JSON files**
(2–3 GB and beyond). Built with **Tauri v2** (Rust backend + web frontend), packaged as a
`.dmg`. The UI takes after [Dadroit](https://dadroit.com/) and
[jsonviewer.app](https://jsonviewer.app/): a dense, syntax‑highlighted tree with a line‑number
gutter, indentation rails, instant search, and system light/dark themes.

<!-- Images served via jsDelivr's CDN so they load everywhere (incl. regions where raw.githubusercontent.com is blocked). -->
![Huge JSON Viewer](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Light theme</summary>

![Huge JSON Viewer — light theme](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Why it can open files that crash other viewers

A 2–3 GB JSON file cannot be parsed into in‑memory objects — parsed JSON balloons to
15–30 GB of RAM. This app never does that. Instead the Rust core:

1. **Memory‑maps** the file (`memmap2`) — the OS pages it in on demand; it is *not* heap.
2. **Streams a single tokenizer pass** over the raw bytes to build a compact, flat
   **structure‑of‑arrays index** (~23 bytes per JSON node). The file is never held as parsed
   objects — only byte offsets and structure.
3. **Renders only visible rows.** The tree is fully virtualized: the backend answers windowed
   `get_rows(start, count)` queries, and expand/collapse mutate a visible‑row list rather than
   materializing the whole tree.
4. **Searches raw bytes** with SIMD substring scanning (`memchr::memmem`) / regex over the
   mmap, mapping each hit back to its node — so search works across the entire file.

### The scrollbar problem (and fix)

A 2–3 GB array can have **millions of visible rows**. At ~22 px/row that exceeds the browser's
maximum element height (~33 M px). A naïve 1:1 scroll spacer would break exactly where it
matters. The frontend **clamps the spacer** and remaps scroll position, so scrolling stays
smooth across millions of rows.

## Features

- Open `.json`, `.ndjson` / `.jsonl` (auto‑detected & wrapped), `.geojson`, `.txt`
- Virtualized collapsible tree with line numbers, indent rails, type‑colored values, child counts
- Background indexing with a live progress bar
- Search keys and/or values, case‑insensitive or case‑sensitive, plain or **regex**, with
  match count and next/prev navigation that auto‑expands ancestors and jumps to the match
- Breadcrumb path of the selected node in the status bar
- Keyboard navigation (↑/↓, PageUp/Down, Home/End, ←/→ collapse/expand, Enter toggle)
- Drag‑and‑drop a file onto the window; ⌘O to open, ⌘F to search
- Light/dark themes following the system appearance

## Languages

The UI ships in **20 locales**, switchable from the 🌐 button in the title bar (persisted, and
auto-detected from your system on first launch). Right-to-left locales (Arabic, Urdu, Punjabi)
mirror the whole interface while keeping the JSON tree left-to-right; numbers are formatted per
locale.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` ·
`id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` ·
`ta-IN` · `tr-TR`

All strings live in [`src/i18n.ts`](src/i18n.ts) — add a locale by appending one dictionary.

## Develop

```bash
npm install
npm run tauri dev      # hot-reloading dev app
```

## Build the DMG

```bash
npm run tauri build -- --bundles dmg
# → src-tauri/target/release/bundle/dmg/*.dmg
```

The app is unsigned by default. To distribute it, sign with a Developer ID certificate and
notarize (`xcrun notarytool submit --wait` + `xcrun stapler staple`). For local use, open the
`.app`/`.dmg` via right‑click → Open the first time to bypass Gatekeeper.

## Architecture

| Piece | File |
| --- | --- |
| Streaming tokenizer, flat index, virtual‑scroll ops, search | `src-tauri/src/index.rs` |
| Tauri commands, background indexing, IPC state | `src-tauri/src/lib.rs` |
| IPC DTOs (serde) | `src-tauri/src/model.rs` |
| Typed IPC wrappers | `src/ipc.ts` |
| UI controller + virtual scroll | `src/main.ts` |
| Design system / theming | `src/styles.css` |

Run the engine tests (tokenizer vs. a `serde_json` oracle, escapes, NDJSON, search, reveal):

```bash
cd src-tauri && cargo test
```

## Current limits (v1)

- **File size:** up to 4 GB (byte offsets are `u32` for compactness). Larger files are rejected
  with a clear message; a `u64`‑offset rebuild would lift this.
- **Node count / RAM:** capped at 300 M JSON nodes. The flat index is ~23 bytes/node, so a
  typical 2–3 GB file (~13 source‑bytes/node) needs roughly **1.5–2× the file size in RAM** for
  the index (the memory‑mapped file itself is evictable OS page cache, not heap). A 16 GB Mac
  comfortably handles 2–3 GB files. Only pathological inputs (e.g. a 3 GB array of billions of
  single‑digit numbers) hit the cap and are reported gracefully.
- **Search** matches **raw file bytes** — an escaped character (`A`, `\n`) matches only in
  its escaped form; case‑insensitive matching is ASCII‑only.
- macOS / Apple Silicon build. (Tauri is cross‑platform; only packaging/QA here targets macOS.)
