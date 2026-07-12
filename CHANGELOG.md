# Changelog

All notable changes to **BigJSON** (formerly Huge JSON Viewer) are documented
here. The format follows [Keep a Changelog](https://keepachangelog.com/), and the
project adheres to [Semantic Versioning](https://semver.org/).

## [0.5.1] — 2026-07-12

### Added
- **Anonymous, opt-out usage analytics** to help prioritize what to build next.
  It **never** sends file names, paths, search queries, or any file content —
  only anonymous events (which features get used, a coarse file-size *bucket*,
  platform, version) with a random per-install id. A first-run notice and a
  one-click opt-out are built in, and it is disabled entirely in builds without
  analytics keys.

### Changed
- Reworded the privacy claims to be accurate: your files never leave your
  device; the only thing sent is anonymous, opt-out usage stats.

## [0.5.0] — 2026-07-11

Clipboard & extract features, plus a round of engine performance work.

### Added
- **Copy to clipboard** — right-click any tree row to **Copy key**, **Copy
  value**, **Copy as JSON** (the node's raw subtree), or **Copy path** (jq-style,
  e.g. `.users[3].name`). Large values are capped at 16 MB with a hint to export.
- **Export subtree as JSON** — the Export menu and the right-click menu can now
  save the whole document *or* the selected node as JSON, streamed straight from
  the source bytes (any size, constant memory).
- **Paste JSON to open** — a *Paste JSON* button on the start screen and
  **⌘V / Ctrl+V** open JSON straight from the clipboard.
- **Update notice** — a subtle "Update available" link appears in the status bar
  when a newer release exists (checked at most once a day; fully offline-safe).

### Performance
- **Search** — replaced the per-hit binary search in match classification with a
  monotonic cursor (removes an O(log N) cache-miss lookup per raw hit), and the
  default case-insensitive search now uses an aho-corasick SIMD matcher instead
  of the regex engine. Case-insensitive **literal** search now folds ASCII case
  (regex search remains Unicode-aware).
- **Indexing** — the hottest string scanner now uses SIMD (`memchr2`), and the
  node-array reservation is sized for real record files to avoid a multi-GB
  reallocation mid-build.
- **Navigation** — jumping to an already-visible search match no longer rebuilds
  the whole visible-row list; the expanded-set now uses a faster hasher.
- **Export** — CSV/XML writers batch runs of bytes instead of writing one byte
  or space at a time.

## [0.4.0] — 2026-07-10

**Windows support** — BigJSON now ships for **Windows 10/11 (x64)** alongside
macOS, with the same engine, UI and features.

### Added
- **Windows build** — a per-user NSIS installer (`.exe`, no admin required),
  built in CI on a Windows runner. Same tree viewer, key & value RegEx search,
  JSON → CSV / XML export, and multi-file union as the macOS build.
- The release workflow now produces both the macOS universal DMG and the
  Windows installer for each tag, and CI builds & tests on macOS **and** Windows.

### Changed
- **Per-OS window chrome** — Windows uses its native title bar (the macOS
  transparent title bar and traffic-light spacing are macOS-only); the open
  shortcut hint shows `Ctrl` on Windows and `⌘` on macOS.
- Website and all READMEs updated to cover macOS **and** Windows (dual download
  buttons, Windows install notes, a Windows FAQ).

### Fixed
- **Multi-file union on Windows** — the temporary scratch file is now created
  with `FILE_FLAG_DELETE_ON_CLOSE` so it auto-cleans after its memory-map closes
  (Windows cannot delete an open file the way Unix can).
- Gated the Unix-only `mmap.advise()` prefetch hints behind `#[cfg(unix)]` so
  the Windows build compiles.

## [0.3.0] — 2026-07-10

### Changed
- **Renamed to BigJSON** (full name: *BigJSON — Huge-size JSON Viewer GUI*),
  formerly **Huge JSON Viewer**, to avoid a name clash with other tools. The
  window title, app name, website and docs now read **BigJSON**; the app bundle
  identifier is `com.bigjson.viewer`. The GitHub repository slug and Rust/npm
  package names stay `huge-json-viewer` so existing links keep working.

## [0.2.0] — 2026-07-09

Free parity with the features **Dadroit** charges for.

### Added
- **Convert JSON → CSV** — streaming export of the whole document or a selected
  node. An array of objects becomes a spreadsheet: columns are auto-discovered
  (union of keys, first-seen order), with RFC-4180 quoting, a UTF-8 BOM and CRLF
  for Excel, CSV formula-injection guarding, and full-precision numbers. Runs on
  multi-GB files with bounded memory (validated at 2,000,000 rows → 246 MB in ~3.5 s).
- **Convert JSON → XML** — streaming export with valid XML 1.0 element-name
  sanitization (original key preserved as a `key="…"` attribute when changed),
  arrays as repeated `<item>`, and iterative walking (no stack overflow on deep JSON).
- **Union multiple files** — open several JSON files at once as one combined,
  searchable tree. Each file is labeled by its filename; search, breadcrumb and
  export all work across the union.
- **Export UI** — Export menu (whole document / selection), native save dialog,
  live progress with **Cancel**, a success toast with **Reveal in Finder**, and a
  skipped-files banner for unions.
- **Multi-file open** — multi-select in the Open dialog or drag several files onto
  the window to union them.
- In-app note: **free for commercial use (MIT) — nothing to license**.

### Fixed
- Nine correctness edge cases found by an adversarial code review:
  - XML element names now use real XML 1.0 NameChar ranges (keys like `a×b`, `a→b`
    are sanitized instead of emitting non-well-formed XML).
  - CSV nested-JSON cells truncate on a UTF-8 char boundary (never split a
    multi-byte character).
  - Numbers are emitted in full (no silent truncation of long number tokens).
  - XML text/attributes escape the noncharacters U+FFFE/U+FFFF.
  - The union scratch file and partial export files are cleaned up on every error path.
  - Breadcrumb tolerates an out-of-range node id instead of panicking.
  - A concurrent file-open is ignored while an export is in flight.

### Notes
- Still free & open source (MIT). Matches every Dadroit paid tier **except** literal
  1 TB files — that needs an on-disk index and remains out of scope; the app supports
  up to **4 GB** per file (and 4 GB combined for a union).

## [0.1.0] — 2026-07-09

Initial release.

### Added
- Open and search **very large JSON files (up to 4 GB)** on macOS in seconds —
  a free, open-source alternative to Dadroit.
- Memory-mapped streaming index; **virtualized**, syntax-highlighted collapsible
  tree with a line-number gutter (Dadroit / jsonviewer.app style), sized to scroll
  past the browser element-height limit.
- **Search keys and values** — case / regex, live match count, next/prev with
  jump-to-match that auto-expands the path.
- Opens `.json`, `.ndjson` / `.jsonl` (auto-detected), `.geojson`, `.txt`.
- **20-language UI**, right-to-left aware (Arabic, Urdu, Punjabi).
- Universal build — Apple Silicon **and** Intel. 100% offline, no telemetry.

[0.2.0]: https://github.com/bandusix/huge-json-viewer/releases/tag/v0.2.0
[0.1.0]: https://github.com/bandusix/huge-json-viewer/releases/tag/v0.1.0
