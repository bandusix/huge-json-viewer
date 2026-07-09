# Huge JSON Viewer

> Open and search **very large JSON files (2–3 GB and up)** on **macOS** in seconds — a **free, open‑source alternative to Dadroit**. When your text editor or browser crashes on a big JSON file, this opens it instantly.

**English** · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)

### [⬇️ Download the latest DMG](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Changelog](CHANGELOG.md)

![Huge JSON Viewer](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Light theme</summary>

![Huge JSON Viewer — light theme](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Why this exists

Most text editors and online JSON viewers **crash or freeze when you open a large JSON file**, because they parse the whole thing into memory — a 2–3 GB file balloons to 15–30 GB of RAM. **Huge JSON Viewer** never does that. It memory‑maps the file, builds a compact index in one streaming pass, and renders only the rows on screen. So it **opens multi‑gigabyte JSON in seconds** and searches the whole file instantly, staying under ~1.5–2× the file size in RAM.

If you've searched for *"how to open a large JSON file"*, *"JSON file too big to open"*, or a **free Dadroit alternative for Mac**, this is built for exactly that.

## Features

- ⚡ **Opens 2–3 GB JSON in ~3 seconds** — memory‑mapped, streaming index, virtualized tree
- 🔍 **Search keys and values** — case‑sensitive or not, plain or **regex**, live match count, next/prev with jump‑to‑match that auto‑expands the path
- 📤 **Convert JSON → CSV or XML** — streaming export that works on multi‑GB files (a huge array of objects becomes a spreadsheet)
- 🔗 **Union multiple files** — open several JSON files at once as one combined, searchable tree
- 🌳 **Syntax‑highlighted collapsible tree** with line numbers, indent guides, type colors and child counts (Dadroit / jsonviewer.app style)
- 📄 Opens `.json`, `.ndjson` / `.jsonl` (auto‑detected), `.geojson`, `.txt`
- 🖱️ Drag‑and‑drop, ⌘O to open, ⌘F to search, full keyboard navigation
- 🌍 **20‑language UI**, right‑to‑left aware (Arabic, Urdu, Punjabi)
- 🖥️ **Universal** — Apple Silicon (M1–M4) *and* Intel
- 🔒 **100% offline** — no upload, no server, no telemetry · 2 MB app
- 🆓 **Free and open source** (MIT)

## Install

1. **[Download the latest `.dmg`](https://github.com/bandusix/huge-json-viewer/releases/latest)** and open it.
2. Drag **Huge JSON Viewer** into Applications.
3. First launch: the app is unsigned, so **right‑click the app → Open**, then confirm (only needed once).

Requirements: macOS 11 (Big Sur) or newer. Handles JSON files up to 4 GB.

## How it works

A 2–3 GB JSON file cannot be parsed into in‑memory objects. The Rust core instead:

1. **Memory‑maps** the file (`memmap2`) — paged in on demand by the OS, not held on the heap.
2. **Streams a single tokenizer pass** to build a compact flat index (~23 bytes per JSON node) of byte offsets and structure — never parsed objects.
3. **Renders only visible rows.** The tree is fully virtualized; expand/collapse mutate a visible‑row list instead of materializing the whole document. A **scaled scrollbar** keeps millions of rows scrollable past the browser's element‑height limit.
4. **Searches raw bytes** with SIMD substring / regex over the mmap and maps every hit back to its node.

Built with **Tauri v2** (Rust backend + web frontend), packaged as a ~2 MB `.dmg`.

## Huge JSON Viewer vs. other large‑JSON tools

| | Huge JSON Viewer | Dadroit | Text editors (VS Code, etc.) |
| --- | --- | --- | --- |
| Price | **Free & open source (MIT)** | Free + $98–198/yr Pro | Free |
| Opens 2–3 GB JSON | ✅ ~3 s | ✅ (2 GB Standard) | ❌ crashes / freezes |
| RAM for a 3 GB file | **~1.5–2×** | low | often out‑of‑memory |
| Search keys **and** values | ✅ regex | ✅ | limited |
| Convert JSON → CSV / XML | ✅ streaming | ✅ | ❌ |
| Union multiple files | ✅ | ✅ (Advanced tier) | ❌ |
| Commercial use | ✅ **free** | 💲 paid license | ✅ |
| Native macOS (Apple Silicon + Intel) | ✅ universal | ✅ | ✅ |
| UI languages | **20 (RTL aware)** | few | many |
| Offline / no telemetry | ✅ | ✅ | ✅ |

## Languages

The UI ships in **20 locales**, switchable from the 🌐 button (persisted, auto‑detected on first launch). Right‑to‑left locales mirror the interface while keeping the JSON tree left‑to‑right; numbers format per locale.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Build from source

```bash
npm install
npm run tauri dev                      # hot-reloading dev app
npm run tauri build -- --bundles dmg   # build the DMG
cd src-tauri && cargo test             # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Limits (v1)

- **File size:** up to 4 GB (compact `u32` offsets). Larger files are rejected with a clear message.
- **RAM:** the index is ~23 bytes/node, so a 2–3 GB file needs roughly **1.5–2× the file size** in RAM (the memory‑mapped file itself is evictable OS page cache). A 16 GB Mac handles 2–3 GB files comfortably.
- **Search** matches raw file bytes (an escaped character matches in its escaped form); case‑insensitive matching is ASCII‑only.

## License

[MIT](LICENSE) © bandusix

<sub>Keywords: large JSON viewer, open big JSON file, view 2GB / 3GB JSON, JSON file too big to open, macOS JSON viewer, native Mac JSON viewer, free Dadroit alternative, open source JSON viewer, search large JSON keys and values, gigabyte JSON, streaming JSON viewer.</sub>
