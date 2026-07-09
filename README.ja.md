# Huge JSON Viewer

> **大きいJSONファイル（2〜3 GB以上）**を **macOS** で数秒で開いて検索。**無料・オープンソースの Dadroit 代替**です。テキストエディタやブラウザが巨大なJSONファイルでクラッシュするとき、これなら一瞬で開けます。

[English](README.md) · [简体中文](README.zh-CN.md) · **日本語** · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)

### [⬇️ 最新版DMGをダウンロード](https://github.com/bandusix/huge-json-viewer/releases/latest)

![Huge JSON Viewer — 大容量JSONファイルを開いた画面（ダークテーマ）](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>ライトテーマ</summary>

![Huge JSON Viewer — ライトテーマ](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## なぜこのアプリを作ったのか

多くのテキストエディタやオンラインのJSONビューアは、**大容量JSONファイルを開こうとするとクラッシュしたりフリーズしたり**します。ファイル全体をメモリに読み込んでパースするため、2〜3 GBのファイルが15〜30 GBのRAMに膨れ上がってしまうのです。**Huge JSON Viewer** はそんなことをしません。ファイルをメモリマップし、ストリーミングで一度だけ走査してコンパクトなインデックスを構築し、画面に表示される行だけを描画します。だから**ギガバイト級のJSONを数秒で開き**、ファイル全体を瞬時に検索でき、RAM使用量はファイルサイズの約1.5〜2倍に収まります。

「**大きいJSONファイル 開く**」「**JSONファイルが大きすぎて開けない**」「**Mac向けの無料 Dadroit 代替**」といったキーワードで検索したことがあるなら、まさにそのために作られたアプリです。

## 主な機能

- ⚡ **2〜3 GBのJSONを約3秒で開く** — メモリマップ、ストリーミングインデックス、仮想化ツリー
- 🔍 **キーと値を検索** — 大文字小文字の区別あり／なし、通常検索または**正規表現**、リアルタイムのマッチ件数、次へ／前へジャンプ（一致箇所へのパスを自動展開）
- 🌳 **シンタックスハイライト付きの折りたたみ可能なツリー** — 行番号、インデントガイド、型ごとの色分け、子要素の件数を表示（Dadroit / jsonviewer.app スタイル）
- 📄 `.json`、`.ndjson` / `.jsonl`（自動判定）、`.geojson`、`.txt` に対応
- 🖱️ ドラッグ＆ドロップ、⌘Oで開く、⌘Fで検索、フルキーボード操作
- 🌍 **20言語のUI**、右横書き（RTL）対応（アラビア語、ウルドゥー語、パンジャブ語）
- 🖥️ **ユニバーサル対応** — Apple Silicon（M1〜M4）*と* Intel の両方
- 🔒 **100%オフライン** — アップロードなし、サーバーなし、テレメトリなし・アプリサイズ2 MB
- 🆓 **無料・オープンソース**（MIT）

## インストール

1. **[最新版の `.dmg` をダウンロード](https://github.com/bandusix/huge-json-viewer/releases/latest)** して開きます。
2. **Huge JSON Viewer** をアプリケーションフォルダにドラッグします。
3. 初回起動時：このアプリは未署名のため、**アプリを右クリック → 開く** を選び、確認してください（初回のみ）。

動作環境：macOS 11（Big Sur）以降。最大4 GBまでのJSONファイルに対応します。

## 仕組み

2〜3 GBのJSONファイルは、メモリ上のオブジェクトとしてパースすることはできません。そこでRust製のコアエンジンは、次のように動作します。

1. ファイルを**メモリマップ**します（`memmap2`）— OSが必要に応じてページ単位で読み込み、ヒープには保持しません。
2. **トークナイザで一度だけストリーミング走査**し、バイトオフセットと構造を表すコンパクトなフラットインデックス（JSONノード1つあたり約23バイト）を構築します — オブジェクトとしてパースはしません。
3. **表示中の行だけを描画します。** ツリーは完全に仮想化されており、展開／折りたたみは文書全体を実体化するのではなく、表示行リストを更新するだけです。**スケーリングされたスクロールバー**により、ブラウザの要素高さ制限を超える数百万行でもスクロールできます。
4. mmap上でSIMDによる部分文字列／正規表現検索を行い、**生バイト列を検索**して、すべてのヒットを対応するノードにマッピングします。

**Tauri v2**（Rustバックエンド + Webフロントエンド）で構築し、約2 MBの `.dmg` としてパッケージ化しています。

## Huge JSON Viewer と他の大容量JSONツールの比較

| | Huge JSON Viewer | Dadroit | テキストエディタ（VS Code など） |
| --- | --- | --- | --- |
| 価格 | **無料・オープンソース（MIT）** | 無料 + 有料Pro版 | 無料 |
| 2〜3 GBのJSONを開く | ✅ 約3秒 | ✅ | ❌ クラッシュ／フリーズ |
| 3 GBファイルのRAM使用量 | **約1.5〜2倍** | 低い | メモリ不足になることが多い |
| キー**と**値の検索 | ✅ 正規表現対応 | ✅ | 限定的 |
| ネイティブmacOS（Apple Silicon + Intel） | ✅ ユニバーサル | ✅ | ✅ |
| UI言語 | **20言語（RTL対応）** | 少数 | 多数 |
| オフライン／テレメトリなし | ✅ | ✅ | ✅ |

## 対応言語

UIは**20のロケール**で提供され、🌐ボタンから切り替えられます（設定は保存され、初回起動時に自動判定されます）。右横書きのロケールではインターフェースをミラーリングしつつ、JSONツリーは左横書きのまま維持し、数値はロケールに応じてフォーマットされます。

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## ソースからビルド

```bash
npm install
npm run tauri dev                      # hot-reloading dev app
npm run tauri build -- --bundles dmg   # build the DMG
cd src-tauri && cargo test             # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## 制限事項（v1）

- **ファイルサイズ：** 最大4 GBまで（コンパクトな `u32` オフセットのため）。これより大きいファイルは、わかりやすいメッセージとともに拒否されます。
- **RAM：** インデックスは1ノードあたり約23バイトのため、2〜3 GBのファイルにはおよそ**ファイルサイズの1.5〜2倍**のRAMが必要です（メモリマップされたファイル自体は、OSが破棄可能なページキャッシュです）。16 GBのMacなら2〜3 GBのファイルを余裕で扱えます。
- **検索**はファイルの生バイト列に対して行われます（エスケープされた文字はエスケープされた形で一致します）。大文字小文字を区別しないマッチングはASCIIのみ対応です。

## ライセンス

[MIT](LICENSE) © bandusix

<sub>キーワード：大きいJSONファイル 開く, 巨大JSON ビューア, 大容量JSONファイル, JSONファイル 開けない, macOS JSONビューア, 無料 Dadroit 代替, オープンソース JSONビューア, 大きいJSON 検索, ギガバイト JSON, ストリーミング JSONビューア。</sub>
