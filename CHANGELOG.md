# Changelog

All notable changes to **Huge JSON Viewer** are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/), and the project adheres
to [Semantic Versioning](https://semver.org/).

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
