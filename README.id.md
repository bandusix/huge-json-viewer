# Huge JSON Viewer

> Buka dan cari **file JSON besar (2–3 GB ke atas)** di **macOS** dalam hitungan detik — **penampil JSON besar** yang menjadi **alternatif Dadroit gratis dan open source**. Saat editor teks atau browser Anda crash membuka file JSON yang terlalu besar, aplikasi ini membukanya seketika.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · **Bahasa Indonesia**

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)

### [⬇️ Unduh DMG terbaru](https://github.com/bandusix/huge-json-viewer/releases/latest)

![Huge JSON Viewer](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Tema terang</summary>

![Huge JSON Viewer — tema terang](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Mengapa aplikasi ini ada

Sebagian besar editor teks dan penampil JSON online **crash atau membeku saat Anda membuka file JSON besar**, karena mereka mengurai seluruh isinya ke dalam memori — file 2–3 GB membengkak menjadi 15–30 GB RAM. **Huge JSON Viewer** tidak pernah melakukan itu. Aplikasi ini memetakan file ke memori (memory‑map), membangun indeks ringkas dalam satu kali proses streaming, dan hanya merender baris yang tampil di layar. Jadi aplikasi ini **membuka JSON berukuran gigabyte dalam hitungan detik** dan mencari di seluruh file secara instan, sambil tetap menggunakan RAM di bawah ~1,5–2× ukuran file.

Jika Anda pernah mencari *"cara membuka file JSON besar"*, *"file JSON terlalu besar untuk dibuka"*, atau **penampil JSON untuk Mac alternatif Dadroit gratis**, aplikasi inilah yang dibuat tepat untuk itu.

## Fitur

- ⚡ **Membuka JSON 2–3 GB dalam ~3 detik** — memory‑map, indeks streaming, pohon tervirtualisasi
- 🔍 **Cari kunci dan nilai** — peka atau tidak peka huruf besar/kecil, teks biasa atau **regex**, jumlah kecocokan langsung, berikutnya/sebelumnya dengan lompat‑ke‑kecocokan yang otomatis membuka jalurnya
- 🌳 **Pohon lipat dengan penyorotan sintaks** lengkap dengan nomor baris, panduan indentasi, warna tipe, dan jumlah anak (gaya Dadroit / jsonviewer.app)
- 📄 Membuka `.json`, `.ndjson` / `.jsonl` (terdeteksi otomatis), `.geojson`, `.txt`
- 🖱️ Seret‑dan‑lepas, ⌘O untuk membuka, ⌘F untuk mencari, navigasi keyboard penuh
- 🌍 **Antarmuka 20 bahasa**, mendukung kanan‑ke‑kiri (Arab, Urdu, Punjabi)
- 🖥️ **Universal** — Apple Silicon (M1–M4) *dan* Intel
- 🔒 **100% offline** — tanpa unggahan, tanpa server, tanpa telemetri · aplikasi 2 MB
- 🆓 **Gratis dan open source** (MIT)

## Instalasi

1. **[Unduh `.dmg` terbaru](https://github.com/bandusix/huge-json-viewer/releases/latest)** lalu buka.
2. Seret **Huge JSON Viewer** ke folder Applications.
3. Peluncuran pertama: aplikasi belum ditandatangani, jadi **klik kanan aplikasi → Open**, lalu konfirmasi (hanya perlu sekali).

Persyaratan: macOS 11 (Big Sur) atau lebih baru. Menangani file JSON hingga 4 GB.

## Cara kerjanya

File JSON 2–3 GB tidak bisa diurai menjadi objek di dalam memori. Sebagai gantinya, inti Rust:

1. **Memetakan file ke memori** (`memmap2`) — dimuat sesuai kebutuhan oleh OS, tidak disimpan di heap.
2. **Menjalankan satu kali proses tokenizer secara streaming** untuk membangun indeks datar ringkas (~23 byte per node JSON) berisi offset byte dan struktur — bukan objek yang diurai.
3. **Hanya merender baris yang terlihat.** Pohonnya sepenuhnya tervirtualisasi; buka/tutup mengubah daftar baris yang terlihat alih‑alih memuat seluruh dokumen. **Scrollbar berskala** menjaga jutaan baris tetap bisa digulir melewati batas tinggi elemen browser.
4. **Mencari byte mentah** dengan substring / regex SIMD di atas mmap dan memetakan setiap kecocokan kembali ke node‑nya.

Dibangun dengan **Tauri v2** (backend Rust + frontend web), dikemas sebagai `.dmg` berukuran ~2 MB.

## Huge JSON Viewer vs. alat JSON besar lainnya

| | Huge JSON Viewer | Dadroit | Editor teks (VS Code, dll.) |
| --- | --- | --- | --- |
| Harga | **Gratis & open source (MIT)** | Gratis + Pro berbayar | Gratis |
| Membuka JSON 2–3 GB | ✅ ~3 dtk | ✅ | ❌ crash / membeku |
| RAM untuk file 3 GB | **~1,5–2×** | rendah | sering kehabisan memori |
| Cari kunci **dan** nilai | ✅ regex | ✅ | terbatas |
| macOS native (Apple Silicon + Intel) | ✅ universal | ✅ | ✅ |
| Bahasa antarmuka | **20 (mendukung RTL)** | sedikit | banyak |
| Offline / tanpa telemetri | ✅ | ✅ | ✅ |

## Bahasa

Antarmuka tersedia dalam **20 lokal**, dapat diganti dari tombol 🌐 (tersimpan, terdeteksi otomatis saat peluncuran pertama). Lokal kanan‑ke‑kiri mencerminkan antarmuka sambil menjaga pohon JSON tetap kiri‑ke‑kanan; angka diformat sesuai lokal.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Build dari sumber

```bash
npm install
npm run tauri dev                      # hot-reloading dev app
npm run tauri build -- --bundles dmg   # build the DMG
cd src-tauri && cargo test             # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Batasan (v1)

- **Ukuran file:** hingga 4 GB (offset `u32` ringkas). File yang lebih besar ditolak dengan pesan yang jelas.
- **RAM:** indeksnya ~23 byte/node, jadi file 2–3 GB membutuhkan sekitar **1,5–2× ukuran file** dalam RAM (file yang dipetakan ke memori itu sendiri adalah cache halaman OS yang bisa dibuang). Mac dengan RAM 16 GB menangani file 2–3 GB dengan nyaman.
- **Pencarian** mencocokkan byte mentah file (karakter ter‑escape cocok dalam bentuk ter‑escape‑nya); pencocokan tanpa peka huruf besar/kecil hanya untuk ASCII.

## Lisensi

[MIT](LICENSE) © bandusix

<sub>Kata kunci: penampil JSON besar, buka file JSON besar, buka JSON 2GB/3GB, file JSON terlalu besar untuk dibuka, penampil JSON untuk Mac, alternatif Dadroit gratis, penampil JSON open source, cari di JSON besar, JSON gigabyte, penampil JSON streaming.</sub>
