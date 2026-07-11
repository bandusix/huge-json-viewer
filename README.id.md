# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> Buka dan cari **file JSON besar (2–3 GB ke atas)** di **macOS dan Windows** dalam hitungan detik — **penampil JSON besar** yang menjadi **alternatif Dadroit gratis dan open source**. Saat editor teks atau browser Anda crash membuka file JSON yang terlalu besar, aplikasi ini membukanya seketika.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · **Bahasa Indonesia**

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ Unduh untuk macOS atau Windows](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Changelog](CHANGELOG.md)

![BigJSON](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Tema terang</summary>

![BigJSON — tema terang](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Mengapa aplikasi ini ada

Sebagian besar editor teks dan penampil JSON online **crash atau membeku saat Anda membuka file JSON besar**, karena mereka mengurai seluruh isinya ke dalam memori — file 2–3 GB membengkak menjadi 15–30 GB RAM. **BigJSON** tidak pernah melakukan itu. Aplikasi ini memetakan file ke memori (memory‑map), membangun indeks ringkas dalam satu kali proses streaming, dan hanya merender baris yang tampil di layar. Jadi aplikasi ini **membuka JSON berukuran gigabyte dalam hitungan detik** dan mencari di seluruh file secara instan, sambil tetap menggunakan RAM di bawah ~1,5–2× ukuran file.

Jika Anda pernah mencari *"cara membuka file JSON besar"*, *"file JSON terlalu besar untuk dibuka"*, atau **alternatif Dadroit gratis untuk Mac atau Windows**, aplikasi inilah yang dibuat tepat untuk itu.

## Fitur

- ⚡ **Membuka JSON 2–3 GB dalam ~3 detik** — memory‑map, indeks streaming, pohon tervirtualisasi
- 🔍 **Cari kunci dan nilai** — peka atau tidak peka huruf besar/kecil, teks biasa atau **regex**, jumlah kecocokan langsung, berikutnya/sebelumnya dengan lompat‑ke‑kecocokan yang otomatis membuka jalurnya
- 📤 **Konversi JSON → CSV atau XML** — ekspor streaming yang bekerja pada file berukuran multi‑GB (array objek yang sangat besar menjadi lembar kerja)
- 📋 **Salin & ekstrak** — klik kanan node mana pun untuk **menyalin kunci, nilai, jalurnya** (gaya jq, mis. `.users[3].name`), atau **menyalin / mengekspor subtree sebagai JSON**
- 🔗 **Gabungkan beberapa file** — buka beberapa file JSON sekaligus sebagai satu pohon gabungan yang dapat dicari
- 🌳 **Pohon lipat dengan penyorotan sintaks** lengkap dengan nomor baris, panduan indentasi, warna tipe, dan jumlah anak (gaya Dadroit / jsonviewer.app)
- 📄 Membuka `.json`, `.ndjson` / `.jsonl` (terdeteksi otomatis), `.geojson`, `.txt` — atau **tempel JSON** langsung dari clipboard
- 🖱️ Seret‑dan‑lepas, **⌘O / Ctrl+O** untuk membuka, **⌘V / Ctrl+V** untuk menempel, **⌘F / Ctrl+F** untuk mencari, navigasi keyboard penuh
- 🔔 **Pemberitahuan pembaruan** — tautan halus di bilah status saat versi baru dirilis (diperiksa paling banyak sekali sehari; sepenuhnya aman offline)
- 🌍 **Antarmuka 20 bahasa**, mendukung kanan‑ke‑kiri (Arab, Urdu, Punjabi)
- 🖥️ **macOS & Windows** — Mac universal (Apple Silicon M1–M4 + Intel) dan Windows 10/11 (x64)
- 🔒 **100% offline** — tanpa unggahan, tanpa server, tanpa telemetri · aplikasi 2 MB
- 🆓 **Gratis dan open source** (MIT)

## Instalasi

### macOS

1. **[Unduh `.dmg` terbaru](https://github.com/bandusix/huge-json-viewer/releases/latest)** lalu buka.
2. Seret **BigJSON** ke folder Applications.
3. Peluncuran pertama: aplikasi belum ditandatangani, jadi **klik kanan aplikasi → Open**, lalu konfirmasi (hanya perlu sekali).

Persyaratan: macOS 11 (Big Sur) atau lebih baru, Apple Silicon atau Intel.

### Windows

1. **[Unduh `.exe` terbaru](https://github.com/bandusix/huge-json-viewer/releases/latest)** (installer NSIS `BigJSON_x.y.z_x64-setup.exe`) lalu jalankan — terpasang per‑pengguna, tanpa perlu admin.
2. Build ini belum ditandatangani, jadi jika **SmartScreen** muncul, klik **More info → Run anyway** (hanya perlu sekali).

Persyaratan: Windows 10 atau 11 (64‑bit). WebView2 sudah terpasang di Windows 11 dan Windows 10 versi terkini; installer mengambilnya secara otomatis jika belum ada.

Kedua build menangani file JSON hingga 4 GB.

## Cara kerjanya

File JSON 2–3 GB tidak bisa diurai menjadi objek di dalam memori. Sebagai gantinya, inti Rust:

1. **Memetakan file ke memori** (`memmap2`) — dimuat sesuai kebutuhan oleh OS, tidak disimpan di heap.
2. **Menjalankan satu kali proses tokenizer secara streaming** untuk membangun indeks datar ringkas (~23 byte per node JSON) berisi offset byte dan struktur — bukan objek yang diurai.
3. **Hanya merender baris yang terlihat.** Pohonnya sepenuhnya tervirtualisasi; buka/tutup mengubah daftar baris yang terlihat alih‑alih memuat seluruh dokumen. **Scrollbar berskala** menjaga jutaan baris tetap bisa digulir melewati batas tinggi elemen browser.
4. **Mencari byte mentah** dengan substring / regex SIMD di atas mmap dan memetakan setiap kecocokan kembali ke node‑nya.

Dibangun dengan **Tauri v2** (backend Rust + frontend web), dikemas sebagai `.dmg` berukuran ~2 MB (macOS) atau installer NSIS `.exe` (Windows).

## BigJSON vs. Dadroit vs. editor teks

Perbandingan yang faktual, fitur demi fitur. **BigJSON adalah alternatif [Dadroit](https://dadroit.com) yang gratis dan open source:** semua yang ada di bawah ini — file besar, pencarian, ekspor CSV/XML, penggabungan banyak file, penggunaan komersial — tersedia **tanpa biaya**, sedangkan Dadroit mengunci fitur‑fitur ini di balik tingkatan berbayar **$98–$198/yr**.

| | **BigJSON** | **Dadroit** | Editor teks (VS Code, dll.) |
| --- | --- | --- | --- |
| **Harga** | **Gratis & open source (MIT)** | Gratis *non‑komersial, ≤ 50 MB* · **$98/yr** (≤ 2 GB) · **$198/yr** (≤ 1 TB) | gratis / berbayar |
| **Penggunaan komersial** | ✅ **gratis** | 💲 berbayar ($98/yr+) — tingkat gratis bersifat non‑komersial | ✅ |
| **Open source** | ✅ MIT, dapat diaudit | ❌ closed source | campuran |
| **Gratis hingga** | **4 GB** | 50 MB, lalu berbayar | — |
| **Ukuran file maksimum** | 4 GB / file | **1 TB** (tingkat berbayar) | ~beberapa ratus MB sebelum tersendat |
| **Membuka JSON 2–3 GB** | ✅ ~3 dtk | ✅ (tingkat berbayar) | ❌ crash / membeku |
| **Throughput buka mentah** | cepat (~1 GB/s, terbatas CPU) | sangat cepat (klaim vendor ~2 GB/s) | lambat |
| **RAM untuk file 3 GB** | ~1,5–2× (indeks; file yang dipetakan tetap berupa page cache yang bisa dibuang) | ~1× (klaim vendor) | sering kehabisan memori |
| **Penampil pohon lipat** | ✅ | ✅ | ❌ (teks mentah) |
| **Cari kunci & nilai** | ✅ | ✅ | terbatas |
| **Pencarian RegEx** | ✅ | ✅ | ✅ |
| **Konversi → CSV / XML** | ✅ **gratis** | ✅ | ❌ |
| **Ekspor / salin subtree sebagai JSON** | ✅ | ekspor tingkat node | manual |
| **Salin kunci / nilai / jalur (gaya jq)** | ✅ | nilai + ekspor | salin‑tempel manual |
| **Gabungkan beberapa file** | ✅ **gratis** | 💲 tingkat berbayar | ❌ |
| **NDJSON / JSON Lines** | ✅ terdeteksi otomatis | ✅ | ❌ |
| **Segarkan otomatis saat file berubah** | ❌ | ✅ | sebagian |
| **Edit JSON** | ❌ hanya baca | ❌ hanya baca | ✅ |
| **Platform** | macOS (universal) · Windows | Windows · macOS · **Linux** | semua |
| **Bahasa antarmuka** | **20 (mendukung RTL)** | sedikit | banyak |
| **Offline · tanpa telemetri** | ✅ | ✅ | ✅ |
| **Ukuran instalasi** | ~2–5 MB | puluhan MB | — |

**Ringkasnya:** untuk JSON hingga beberapa GB, BigJSON melakukan semua yang dilakukan tingkatan berbayar Dadroit — membuka, mencari dengan regex, mengonversi ke CSV/XML, menggabungkan banyak file, mengekstrak subtree — **secara gratis, open source, tanpa paywall penggunaan komersial atau fitur**. Tingkat gratis Dadroit berhenti di **50 MB** dan melarang penggunaan komersial; membuka kunci 2 GB / komersial biayanya **$98/yr**, dan 1 TB + set fitur lengkap biayanya **$198/yr**.

**Di mana Dadroit masih unggul (secara jujur):** file di atas 4 GB (hingga **1 TB**), build **Linux** native, **penyegaran otomatis** saat file berubah di disk, dan throughput mentah yang lebih tinggi pada benchmark mereka sendiri. Jika Anda rutin membuka file 100 GB–1 TB atau butuh Linux, Dadroit sepadan dengan lisensinya. Untuk semua kebutuhan hingga beberapa gigabyte, BigJSON adalah pilihan yang **gratis dan tanpa batas**.

<sub>Tingkatan dan harga Dadroit menurut [dadroit.com](https://dadroit.com/buy-licence/) (dapat berubah). Angka kecepatan/RAM adalah klaim masing‑masing proyek — lakukan benchmark pada file Anda sendiri untuk membandingkan.</sub>

## Bahasa

Antarmuka tersedia dalam **20 lokal**, dapat diganti dari tombol 🌐 (tersimpan, terdeteksi otomatis saat peluncuran pertama). Lokal kanan‑ke‑kiri mencerminkan antarmuka sambil menjaga pohon JSON tetap kiri‑ke‑kanan; angka diformat sesuai lokal.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Build dari sumber

Prasyarat: [Node.js](https://nodejs.org) 20+, [toolchain Rust](https://rustup.rs), dan [prasyarat sistem Tauri v2](https://v2.tauri.app/start/prerequisites/) untuk OS Anda (Xcode Command Line Tools di macOS; Microsoft C++ Build Tools + WebView2 di Windows).

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Batasan (v1)

- **Ukuran file:** hingga 4 GB (offset `u32` ringkas). File yang lebih besar ditolak dengan pesan yang jelas.
- **RAM:** indeksnya ~23 byte/node, jadi file 2–3 GB membutuhkan sekitar **1,5–2× ukuran file** dalam RAM (file yang dipetakan ke memori itu sendiri adalah cache halaman OS yang bisa dibuang). Mesin dengan RAM 16 GB menangani file 2–3 GB dengan nyaman.
- **Pencarian** mencocokkan byte mentah file (karakter ter‑escape cocok dalam bentuk ter‑escape‑nya); pencocokan tanpa peka huruf besar/kecil hanya untuk ASCII.

## Lisensi

[MIT](LICENSE) © bandusix

<sub>Kata kunci: penampil JSON besar, buka file JSON besar, buka JSON 2GB/3GB, file JSON terlalu besar untuk dibuka, penampil JSON untuk Mac, penampil JSON untuk Windows, penampil JSON native Mac, buka JSON besar di Windows, alternatif Dadroit gratis, penampil JSON open source, cari kunci dan nilai di JSON besar, ekstrak subtree JSON, salin nilai / jalur JSON, JSON ke CSV / XML, penampil NDJSON, JSON gigabyte, penampil JSON streaming.</sub>
