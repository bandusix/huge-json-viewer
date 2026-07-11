# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> **Große JSON-Dateien (2–3 GB und mehr) unter macOS und Windows in Sekunden öffnen und durchsuchen** — eine **kostenlose, quelloffene Alternative zu Dadroit**. Wenn dein Texteditor oder Browser bei einer riesigen JSON-Datei abstürzt, öffnet dieser große JSON-Viewer sie sofort.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · **Deutsch** · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ Für macOS oder Windows herunterladen](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Changelog](CHANGELOG.md)

![BigJSON](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Helles Design</summary>

![BigJSON — helles Design](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Warum es diese App gibt

Die meisten Texteditoren und Online-JSON-Viewer **stürzen ab oder frieren ein, wenn man eine große JSON-Datei öffnet**, weil sie das gesamte Dokument in den Arbeitsspeicher parsen — eine 2–3 GB große Datei bläht sich dabei auf 15–30 GB RAM auf. **BigJSON** macht das nie. Er bildet die Datei per Memory-Mapping ab, erstellt in einem einzigen Streaming-Durchlauf einen kompakten Index und rendert nur die sichtbaren Zeilen. So **öffnet er JSON im Gigabyte-Bereich in Sekunden** und durchsucht die ganze Datei sofort — und bleibt dabei unter etwa dem 1,5- bis 2-Fachen der Dateigröße im RAM.

Wenn du schon einmal nach *„große JSON-Datei öffnen"*, *„JSON-Datei zu groß zum Öffnen"* oder einer **kostenlosen Dadroit-Alternative für Mac oder Windows** gesucht hast, ist dieser JSON-Viewer genau dafür gemacht.

## Funktionen

- ⚡ **Öffnet 2GB/3GB JSON in ~3 Sekunden** — Memory-Mapping, Streaming-Index, virtualisierter Baum
- 🔍 **Schlüssel und Werte durchsuchen** — mit oder ohne Groß-/Kleinschreibung, als Text oder **Regex**, mit Live-Trefferzähler, Weiter/Zurück und Sprung zum Treffer, der den Pfad automatisch aufklappt
- 📤 **JSON → CSV oder XML konvertieren** — Streaming-Export, der auch mit mehreren GB großen Dateien funktioniert (ein riesiges Array von Objekten wird zur Tabelle)
- 📋 **Kopieren & extrahieren** — mit Rechtsklick auf einen beliebigen Knoten **seinen Schlüssel, Wert oder Pfad kopieren** (im jq‑Stil, z. B. `.users[3].name`) oder **einen Teilbaum als JSON kopieren / exportieren**
- 🔗 **Mehrere Dateien vereinen** — mehrere JSON-Dateien gleichzeitig als einen kombinierten, durchsuchbaren Baum öffnen
- 🌳 **Syntaxhervorgehobener, einklappbarer Baum** mit Zeilennummern, Einrückungslinien, Typfarben und Kind-Zählern (im Stil von Dadroit / jsonviewer.app)
- 📄 Öffnet `.json`, `.ndjson` / `.jsonl` (automatisch erkannt), `.geojson`, `.txt` — oder **JSON direkt aus der Zwischenablage einfügen**
- 🖱️ Drag & Drop, **⌘O / Ctrl+O** zum Öffnen, **⌘V / Ctrl+V** zum Einfügen, **⌘F / Ctrl+F** zum Suchen, vollständige Tastaturnavigation
- 🔔 **Update-Hinweis** — ein dezenter Link in der Statusleiste, wenn eine neue Version erscheint (höchstens einmal täglich geprüft; vollständig offline-sicher)
- 🌍 **Oberfläche in 20 Sprachen**, mit Unterstützung für Rechts-nach-links (Arabisch, Urdu, Punjabi)
- 🖥️ **macOS & Windows** — universeller Mac (Apple Silicon M1–M4 + Intel) und Windows 10/11 (x64)
- 🔒 **100 % offline** — kein Upload, kein Server, keine Telemetrie · 2 MB App
- 🆓 **Kostenlos und quelloffen** (MIT)

## Installation

### macOS

1. **[Neueste `.dmg` herunterladen](https://github.com/bandusix/huge-json-viewer/releases/latest)** und öffnen.
2. **BigJSON** in den Ordner „Programme" ziehen.
3. Beim ersten Start: Die App ist nicht signiert, daher **mit Rechtsklick auf die App → Öffnen** starten und bestätigen (nur einmal nötig).

Voraussetzungen: macOS 11 (Big Sur) oder neuer, Apple Silicon oder Intel.

### Windows

1. **[Neueste `.exe` herunterladen](https://github.com/bandusix/huge-json-viewer/releases/latest)** (den NSIS-Installer `BigJSON_x.y.z_x64-setup.exe`) und ausführen — die Installation erfolgt pro Benutzer, ohne Administratorrechte.
2. Der Build ist nicht signiert; erscheint daher **SmartScreen**, auf **Weitere Informationen → Trotzdem ausführen** klicken (nur einmal nötig).

Voraussetzungen: Windows 10 oder 11 (64-Bit). WebView2 ist unter Windows 11 und aktuellem Windows 10 vorinstalliert; der Installer lädt es bei Bedarf automatisch nach.

Beide Builds verarbeiten JSON-Dateien bis zu 4 GB.

## So funktioniert es

Eine 2–3 GB große JSON-Datei lässt sich nicht in speicherinterne Objekte parsen. Der Rust-Kern geht stattdessen so vor:

1. **Memory-Mapping** der Datei (`memmap2`) — vom Betriebssystem bei Bedarf seitenweise eingelagert, nicht auf dem Heap gehalten.
2. **Ein einziger Streaming-Durchlauf des Tokenizers** erstellt einen kompakten, flachen Index (~23 Byte pro JSON-Knoten) aus Byte-Offsets und Struktur — niemals geparste Objekte.
3. **Rendert nur sichtbare Zeilen.** Der Baum ist vollständig virtualisiert; Auf- und Zuklappen verändern eine Liste sichtbarer Zeilen, statt das ganze Dokument zu materialisieren. Ein **skalierter Scrollbalken** hält Millionen Zeilen scrollbar, weit über die Element-Höhengrenze des Browsers hinaus.
4. **Durchsucht die rohen Bytes** per SIMD-Teilstring-/Regex-Suche über das Memory-Mapping und ordnet jeden Treffer wieder seinem Knoten zu.

Entwickelt mit **Tauri v2** (Rust-Backend + Web-Frontend), ausgeliefert als ~2 MB große `.dmg` (macOS) oder als NSIS-`.exe`-Installer (Windows).

## BigJSON im Vergleich zu anderen Werkzeugen für große JSON-Dateien

| | BigJSON | Dadroit | Texteditoren (VS Code usw.) |
| --- | --- | --- | --- |
| Preis | **Kostenlos & quelloffen (MIT)** | Kostenlos + kostenpflichtige Pro-Version | Kostenlos |
| Öffnet 2–3 GB JSON | ✅ ~3 s | ✅ | ❌ stürzt ab / friert ein |
| RAM für eine 3 GB große Datei | **~1,5–2×** | gering | oft Speicherüberlauf |
| Schlüssel **und** Werte durchsuchen | ✅ Regex | ✅ | eingeschränkt |
| JSON → CSV / XML konvertieren | ✅ Streaming | ✅ | ❌ |
| Wert, Pfad, Teilbaum kopieren / extrahieren | ✅ | ✅ | manuell |
| Mehrere Dateien vereinen | ✅ | ✅ (Advanced-Stufe) | ❌ |
| Kommerzielle Nutzung | ✅ **kostenlos** | 💲 kostenpflichtige Lizenz | ✅ |
| Nativ für macOS & Windows | ✅ (universeller Mac + Win x64) | ✅ | ✅ |
| Oberflächensprachen | **20 (RTL-fähig)** | wenige | viele |
| Offline / keine Telemetrie | ✅ | ✅ | ✅ |

## Sprachen

Die Oberfläche wird in **20 Sprachvarianten** ausgeliefert, umschaltbar über die 🌐-Schaltfläche (gespeichert, beim ersten Start automatisch erkannt). Rechts-nach-links-Sprachen spiegeln die Oberfläche, während der JSON-Baum von links nach rechts bleibt; Zahlen werden je nach Sprachraum formatiert.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Aus dem Quellcode bauen

Voraussetzungen: [Node.js](https://nodejs.org) 20+, die [Rust-Toolchain](https://rustup.rs) und die [Tauri-v2-Systemvoraussetzungen](https://v2.tauri.app/start/prerequisites/) für dein Betriebssystem (Xcode Command Line Tools unter macOS; Microsoft C++ Build Tools + WebView2 unter Windows).

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Grenzen (v1)

- **Dateigröße:** bis zu 4 GB (kompakte `u32`-Offsets). Größere Dateien werden mit einer klaren Meldung abgelehnt.
- **RAM:** Der Index umfasst ~23 Byte/Knoten, sodass eine 2–3 GB große Datei etwa das **1,5- bis 2-Fache der Dateigröße** an RAM benötigt (die per Memory-Mapping eingebundene Datei selbst ist verdrängbarer OS-Seitencache). Ein Rechner mit 16 GB verarbeitet 2–3 GB große Dateien mühelos.
- **Die Suche** trifft die rohen Datei-Bytes (ein maskiertes Zeichen wird in seiner maskierten Form gefunden); die Suche ohne Beachtung der Groß-/Kleinschreibung funktioniert nur mit ASCII.

## Lizenz

[MIT](LICENSE) © bandusix

<sub>Schlüsselwörter: großer JSON-Viewer, große JSON-Datei öffnen, 2GB/3GB JSON öffnen, JSON-Datei zu groß, JSON-Viewer für Mac, JSON-Viewer für Windows, große JSON unter Windows öffnen, kostenlose Dadroit-Alternative, Open-Source JSON-Viewer, in großer JSON suchen, JSON-Teilbaum extrahieren, JSON-Wert / -Pfad kopieren, JSON zu CSV / XML, NDJSON-Viewer, Gigabyte JSON, Streaming JSON-Viewer.</sub>
