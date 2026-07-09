# Huge JSON Viewer

> **Çok büyük JSON dosyalarını (2–3 GB ve üzeri)** **macOS** üzerinde saniyeler içinde açın ve içinde arama yapın — **ücretsiz, açık kaynak bir Dadroit alternatifi**. Metin düzenleyiciniz veya tarayıcınız büyük bir JSON dosyasında çökerken, bu uygulama onu anında açar.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · **Türkçe** · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)

### [⬇️ En son DMG'yi indir](https://github.com/bandusix/huge-json-viewer/releases/latest)

![Huge JSON Viewer](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Açık tema</summary>

![Huge JSON Viewer — açık tema](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Bu uygulama neden var

Çoğu metin düzenleyici ve çevrimiçi JSON görüntüleyici, **büyük bir JSON dosyasını açtığınızda çöker veya donar**, çünkü dosyanın tamamını belleğe ayrıştırırlar — 2–3 GB'lık bir dosya 15–30 GB RAM'e şişer. **Huge JSON Viewer** bunu asla yapmaz. Dosyayı belleğe eşler (memory‑map), tek bir akışlı geçişte kompakt bir dizin oluşturur ve yalnızca ekrandaki satırları çizer. Bu akışlı JSON görüntüleyici sayesinde **gigabaytlık JSON dosyalarını saniyeler içinde açar** ve tüm dosya içinde anında arama yapar, RAM'de dosya boyutunun ~1.5–2 katının altında kalır.

*"büyük JSON dosyası nasıl açılır"*, *"JSON dosyası çok büyük açılmıyor"* veya *"Mac için ücretsiz Dadroit alternatifi"* aradıysanız, bu tam da bunun için yapıldı.

## Özellikler

- ⚡ **2–3 GB'lık JSON'u ~3 saniyede açar** — belleğe eşlemeli (memory‑mapped), akışlı dizin, sanallaştırılmış ağaç
- 🔍 **Anahtarlarda ve değerlerde arama** — büyük/küçük harf duyarlı veya değil, düz metin veya **regex**, canlı eşleşme sayısı, yolu otomatik açan eşleşmeye‑atla özelliğiyle ileri/geri
- 🌳 Satır numaraları, girinti kılavuzları, tür renkleri ve alt öğe sayılarıyla **sözdizimi vurgulu, katlanabilir ağaç** (Dadroit / jsonviewer.app tarzı)
- 📄 `.json`, `.ndjson` / `.jsonl` (otomatik algılanır), `.geojson`, `.txt` dosyalarını açar
- 🖱️ Sürükle‑bırak, açmak için ⌘O, aramak için ⌘F, tam klavye navigasyonu
- 🌍 **20 dilli arayüz**, sağdan‑sola (RTL) uyumlu (Arapça, Urduca, Pencapça)
- 🖥️ **Universal** — Apple Silicon (M1–M4) *ve* Intel
- 🔒 **%100 çevrimdışı** — yükleme yok, sunucu yok, telemetri yok · 2 MB'lık uygulama
- 🆓 **Ücretsiz ve açık kaynak** (MIT)

## Kurulum

1. **[En son `.dmg` dosyasını indirin](https://github.com/bandusix/huge-json-viewer/releases/latest)** ve açın.
2. **Huge JSON Viewer**'ı Applications (Uygulamalar) klasörüne sürükleyin.
3. İlk açılış: uygulama imzasızdır, bu yüzden **uygulamaya sağ tıklayın → Aç**, ardından onaylayın (yalnızca bir kez gerekir).

Gereksinimler: macOS 11 (Big Sur) veya üzeri. 4 GB'a kadar JSON dosyalarını işler.

## Nasıl çalışır

2–3 GB'lık bir JSON dosyası bellek içi nesnelere ayrıştırılamaz. Bunun yerine Rust çekirdeği:

1. Dosyayı **belleğe eşler** (`memmap2`) — heap'te tutulmaz, işletim sistemi tarafından talep üzerine sayfalanır.
2. Bayt konumlarının ve yapının kompakt, düz bir dizinini (JSON düğümü başına ~23 bayt) oluşturmak için **tek bir akışlı tokenizer geçişi** yapar — nesneleri asla ayrıştırmaz.
3. **Yalnızca görünür satırları çizer.** Ağaç tamamen sanallaştırılmıştır; genişletme/daraltma, tüm belgeyi somutlaştırmak yerine görünür satır listesini değiştirir. **Ölçeklenmiş bir kaydırma çubuğu**, milyonlarca satırı tarayıcının öğe‑yüksekliği sınırının ötesinde kaydırılabilir tutar.
4. mmap üzerinde SIMD alt dize / regex ile **ham baytlarda arama yapar** ve her eşleşmeyi ilgili düğümüne geri eşler.

**Tauri v2** (Rust arka uç + web ön uç) ile geliştirildi, ~2 MB'lık bir `.dmg` olarak paketlendi.

## Huge JSON Viewer ile diğer büyük JSON araçlarının karşılaştırması

| | Huge JSON Viewer | Dadroit | Metin düzenleyiciler (VS Code vb.) |
| --- | --- | --- | --- |
| Fiyat | **Ücretsiz ve açık kaynak (MIT)** | Ücretsiz + ücretli Pro | Ücretsiz |
| 2–3 GB JSON açar | ✅ ~3 sn | ✅ | ❌ çöker / donar |
| 3 GB dosya için RAM | **~1.5–2×** | düşük | çoğu zaman bellek yetersiz |
| Anahtarlarda **ve** değerlerde arama | ✅ regex | ✅ | sınırlı |
| Yerel macOS (Apple Silicon + Intel) | ✅ universal | ✅ | ✅ |
| Arayüz dilleri | **20 (RTL uyumlu)** | az | çok |
| Çevrimdışı / telemetri yok | ✅ | ✅ | ✅ |

## Diller

Arayüz **20 yerel ayarla** gelir, 🌐 düğmesinden değiştirilebilir (kaydedilir, ilk açılışta otomatik algılanır). Sağdan‑sola yerel ayarlar arayüzü aynalar ancak JSON ağacını soldan‑sağa tutar; sayılar her yerel ayara göre biçimlendirilir.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Kaynaktan derleme

```bash
npm install
npm run tauri dev                      # hot-reloading dev app
npm run tauri build -- --bundles dmg   # build the DMG
cd src-tauri && cargo test             # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Sınırlar (v1)

- **Dosya boyutu:** 4 GB'a kadar (kompakt `u32` konumları). Daha büyük dosyalar açık bir mesajla reddedilir.
- **RAM:** dizin düğüm başına ~23 bayttır, bu yüzden 2–3 GB'lık bir dosya RAM'de kabaca **dosya boyutunun 1.5–2 katına** ihtiyaç duyar (belleğe eşlenen dosyanın kendisi, işletim sisteminin boşaltılabilir sayfa önbelleğidir). 16 GB'lık bir Mac, 2–3 GB'lık dosyaları rahatça işler.
- **Arama** ham dosya baytlarıyla eşleşir (bir kaçış karakteri, kaçış biçiminde eşleşir); büyük/küçük harf duyarsız eşleştirme yalnızca ASCII içindir.

## Lisans

[MIT](LICENSE) © bandusix

<sub>Anahtar kelimeler: büyük JSON dosyası açma, büyük JSON görüntüleyici, 2GB / 3GB JSON açma, JSON dosyası çok büyük, Mac için JSON görüntüleyici, ücretsiz Dadroit alternatifi, açık kaynak JSON görüntüleyici, büyük JSON içinde arama, gigabaytlık JSON, akışlı JSON görüntüleyici.</sub>
