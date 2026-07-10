# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> **Çok büyük JSON dosyalarını (2–3 GB ve üzeri)** **macOS ve Windows** üzerinde saniyeler içinde açın ve içinde arama yapın — **ücretsiz, açık kaynak bir Dadroit alternatifi**. Metin düzenleyiciniz veya tarayıcınız büyük bir JSON dosyasında çökerken, bu uygulama onu anında açar.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · **Türkçe** · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ macOS veya Windows için indir](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Değişiklik günlüğü](CHANGELOG.md)

![BigJSON](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Açık tema</summary>

![BigJSON — açık tema](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Bu uygulama neden var

Çoğu metin düzenleyici ve çevrimiçi JSON görüntüleyici, **büyük bir JSON dosyasını açtığınızda çöker veya donar**, çünkü dosyanın tamamını belleğe ayrıştırırlar — 2–3 GB'lık bir dosya 15–30 GB RAM'e şişer. **BigJSON** bunu asla yapmaz. Dosyayı belleğe eşler (memory‑map), tek bir akışlı geçişte kompakt bir dizin oluşturur ve yalnızca ekrandaki satırları çizer. Bu sayede **gigabaytlık JSON dosyalarını saniyeler içinde açar** ve tüm dosya içinde anında arama yapar, RAM'de dosya boyutunun ~1.5–2 katının altında kalır.

*"büyük JSON dosyası nasıl açılır"*, *"JSON dosyası çok büyük açılmıyor"* veya *"Mac ya da Windows için ücretsiz Dadroit alternatifi"* aradıysanız, bu tam da bunun için yapıldı.

## Özellikler

- ⚡ **2–3 GB'lık JSON'u ~3 saniyede açar** — belleğe eşlemeli (memory‑mapped), akışlı dizin, sanallaştırılmış ağaç
- 🔍 **Anahtarlarda ve değerlerde arama** — büyük/küçük harf duyarlı veya değil, düz metin veya **regex**, canlı eşleşme sayısı, yolu otomatik açan eşleşmeye‑atla özelliğiyle ileri/geri
- 📤 **JSON → CSV veya XML dönüştürme** — çok GB'lık dosyalarda çalışan akışlı dışa aktarma (nesnelerden oluşan devasa bir dizi bir elektronik tabloya dönüşür)
- 🔗 **Birden çok dosyayı birleştirme** — birkaç JSON dosyasını aynı anda tek, birleşik ve aranabilir bir ağaç olarak açın
- 🌳 Satır numaraları, girinti kılavuzları, tür renkleri ve alt öğe sayılarıyla **sözdizimi vurgulu, katlanabilir ağaç** (Dadroit / jsonviewer.app tarzı)
- 📄 `.json`, `.ndjson` / `.jsonl` (otomatik algılanır), `.geojson`, `.txt` dosyalarını açar
- 🖱️ Sürükle‑bırak, açmak için **⌘O / Ctrl+O**, aramak için **⌘F / Ctrl+F**, tam klavye navigasyonu
- 🌍 **20 dilli arayüz**, sağdan‑sola (RTL) uyumlu (Arapça, Urduca, Pencapça)
- 🖥️ **macOS & Windows** — universal Mac (Apple Silicon M1–M4 + Intel) ve Windows 10/11 (x64)
- 🔒 **%100 çevrimdışı** — yükleme yok, sunucu yok, telemetri yok · 2 MB'lık uygulama
- 🆓 **Ücretsiz ve açık kaynak** (MIT)

## Kurulum

### macOS

1. **[En son `.dmg` dosyasını indirin](https://github.com/bandusix/huge-json-viewer/releases/latest)** ve açın.
2. **BigJSON**'ı Applications (Uygulamalar) klasörüne sürükleyin.
3. İlk açılış: uygulama imzasızdır, bu yüzden **uygulamaya sağ tıklayın → Aç**, ardından onaylayın (yalnızca bir kez gerekir).

Gereksinimler: macOS 11 (Big Sur) veya üzeri, Apple Silicon veya Intel.

### Windows

1. **[En son `.exe` dosyasını indirin](https://github.com/bandusix/huge-json-viewer/releases/latest)** (`BigJSON_x.y.z_x64-setup.exe` NSIS yükleyicisi) ve çalıştırın — kullanıcı başına kurulur, yönetici gerektirmez.
2. Derleme imzasızdır, bu yüzden **SmartScreen** görünürse **Ek bilgi → Yine de çalıştır** seçeneğine tıklayın (yalnızca bir kez gerekir).

Gereksinimler: Windows 10 veya 11 (64‑bit). WebView2, Windows 11 ve güncel Windows 10 sürümlerinde önceden yüklüdür; eksikse yükleyici onu otomatik olarak indirir.

Her iki derleme de 4 GB'a kadar JSON dosyalarını işler.

## Nasıl çalışır

2–3 GB'lık bir JSON dosyası bellek içi nesnelere ayrıştırılamaz. Bunun yerine Rust çekirdeği:

1. Dosyayı **belleğe eşler** (`memmap2`) — heap'te tutulmaz, işletim sistemi tarafından talep üzerine sayfalanır.
2. Bayt konumlarının ve yapının kompakt, düz bir dizinini (JSON düğümü başına ~23 bayt) oluşturmak için **tek bir akışlı tokenizer geçişi** yapar — nesneleri asla ayrıştırmaz.
3. **Yalnızca görünür satırları çizer.** Ağaç tamamen sanallaştırılmıştır; genişletme/daraltma, tüm belgeyi somutlaştırmak yerine görünür satır listesini değiştirir. **Ölçeklenmiş bir kaydırma çubuğu**, milyonlarca satırı tarayıcının öğe‑yüksekliği sınırının ötesinde kaydırılabilir tutar.
4. mmap üzerinde SIMD alt dize / regex ile **ham baytlarda arama yapar** ve her eşleşmeyi ilgili düğümüne geri eşler.

**Tauri v2** (Rust arka uç + web ön uç) ile geliştirildi, ~2 MB'lık bir `.dmg` (macOS) veya NSIS `.exe` yükleyicisi (Windows) olarak paketlendi.

## BigJSON ile diğer büyük JSON araçlarının karşılaştırması

| | BigJSON | Dadroit | Metin düzenleyiciler (VS Code vb.) |
| --- | --- | --- | --- |
| Fiyat | **Ücretsiz ve açık kaynak (MIT)** | Ücretsiz + $98–198/yıl Pro | Ücretsiz |
| 2–3 GB JSON açar | ✅ ~3 sn | ✅ (2 GB Standard) | ❌ çöker / donar |
| 3 GB dosya için RAM | **~1.5–2×** | düşük | çoğu zaman bellek yetersiz |
| Anahtarlarda **ve** değerlerde arama | ✅ regex | ✅ | sınırlı |
| JSON → CSV / XML dönüştürme | ✅ akışlı | ✅ | ❌ |
| Birden çok dosyayı birleştirme | ✅ | ✅ (Advanced katman) | ❌ |
| Ticari kullanım | ✅ **ücretsiz** | 💲 ücretli lisans | ✅ |
| Yerel macOS & Windows | ✅ (universal Mac + Win x64) | ✅ | ✅ |
| Arayüz dilleri | **20 (RTL uyumlu)** | az | çok |
| Çevrimdışı / telemetri yok | ✅ | ✅ | ✅ |

## Diller

Arayüz **20 yerel ayarla** gelir, 🌐 düğmesinden değiştirilebilir (kaydedilir, ilk açılışta otomatik algılanır). Sağdan‑sola yerel ayarlar arayüzü aynalar ancak JSON ağacını soldan‑sağa tutar; sayılar her yerel ayara göre biçimlendirilir.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Kaynaktan derleme

Ön koşullar: [Node.js](https://nodejs.org) 20+, [Rust araç zinciri](https://rustup.rs) ve işletim sisteminiz için [Tauri v2 sistem ön koşulları](https://v2.tauri.app/start/prerequisites/) (macOS'ta Xcode Command Line Tools; Windows'ta Microsoft C++ Build Tools + WebView2).

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Sınırlar (v1)

- **Dosya boyutu:** 4 GB'a kadar (kompakt `u32` konumları). Daha büyük dosyalar açık bir mesajla reddedilir.
- **RAM:** dizin düğüm başına ~23 bayttır, bu yüzden 2–3 GB'lık bir dosya RAM'de kabaca **dosya boyutunun 1.5–2 katına** ihtiyaç duyar (belleğe eşlenen dosyanın kendisi, işletim sisteminin boşaltılabilir sayfa önbelleğidir). 16 GB'lık bir makine, 2–3 GB'lık dosyaları rahatça işler.
- **Arama** ham dosya baytlarıyla eşleşir (bir kaçış karakteri, kaçış biçiminde eşleşir); büyük/küçük harf duyarsız eşleştirme yalnızca ASCII içindir.

## Lisans

[MIT](LICENSE) © bandusix

<sub>Anahtar kelimeler: büyük JSON dosyası açma, büyük JSON görüntüleyici, 2GB / 3GB JSON açma, JSON dosyası çok büyük, Mac için JSON görüntüleyici, Windows için JSON görüntüleyici, yerel Mac JSON görüntüleyici, Windows'ta büyük JSON açma, ücretsiz Dadroit alternatifi, açık kaynak JSON görüntüleyici, büyük JSON içinde anahtar ve değer arama, gigabaytlık JSON, akışlı JSON görüntüleyici.</sub>
