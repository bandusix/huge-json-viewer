# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> **大きいJSONファイル（2〜3 GB以上）**を **macOS と Windows** で数秒で開いて検索。**無料・オープンソースの Dadroit 代替**です。テキストエディタやブラウザが巨大なJSONファイルでクラッシュするとき、これなら一瞬で開けます。

[English](README.md) · [简体中文](README.zh-CN.md) · **日本語** · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ macOS または Windows 用をダウンロード](https://github.com/bandusix/huge-json-viewer/releases/latest) · [変更履歴](CHANGELOG.md)

![BigJSON — 大容量JSONファイルを開いた画面（ダークテーマ）](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>ライトテーマ</summary>

![BigJSON — ライトテーマ](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## なぜこのアプリを作ったのか

多くのテキストエディタやオンラインのJSONビューアは、**大容量JSONファイルを開こうとするとクラッシュしたりフリーズしたり**します。ファイル全体をメモリに読み込んでパースするため、2〜3 GBのファイルが15〜30 GBのRAMに膨れ上がってしまうのです。**BigJSON** はそんなことをしません。ファイルをメモリマップし、ストリーミングで一度だけ走査してコンパクトなインデックスを構築し、画面に表示される行だけを描画します。だから**ギガバイト級のJSONを数秒で開き**、ファイル全体を瞬時に検索でき、RAM使用量はファイルサイズの約1.5〜2倍に収まります。

「**大きいJSONファイル 開く**」「**JSONファイルが大きすぎて開けない**」「**Mac や Windows 向けの無料 Dadroit 代替**」といったキーワードで検索したことがあるなら、まさにそのために作られたアプリです。

## 主な機能

- ⚡ **2〜3 GBのJSONを約3秒で開く** — メモリマップ、ストリーミングインデックス、仮想化ツリー
- 🔍 **キーと値を検索** — 大文字小文字の区別あり／なし、通常検索または**正規表現**、リアルタイムのマッチ件数、次へ／前へジャンプ（一致箇所へのパスを自動展開）
- 📤 **JSON → CSV または XML に変換** — マルチGBのファイルでも動作するストリーミングエクスポート（オブジェクトの巨大な配列を表計算データに変換）
- 📋 **コピーと抽出** — 任意のノードを右クリックして、その**キー・値・パス**（jq 形式、例：`.users[3].name`）を**コピー**したり、サブツリーを **JSON としてコピー／エクスポート**できます
- 🔗 **複数ファイルの結合** — 複数のJSONファイルを同時に開き、1つの結合された検索可能なツリーとして表示
- 🌳 **シンタックスハイライト付きの折りたたみ可能なツリー** — 行番号、インデントガイド、型ごとの色分け、子要素の件数を表示（Dadroit / jsonviewer.app スタイル）
- 📄 `.json`、`.ndjson` / `.jsonl`（自動判定）、`.geojson`、`.txt` に対応 — または**クリップボードから JSON を直接貼り付け**
- 🖱️ ドラッグ＆ドロップ、**⌘O / Ctrl+O** で開く、**⌘V / Ctrl+V** で貼り付け、**⌘F / Ctrl+F** で検索、フルキーボード操作
- 🔔 **アップデート通知** — 新しいバージョンが公開されると、ステータスバーに控えめなリンクが表示されます（チェックは1日1回まで、完全にオフラインでも安全）
- 🌍 **20言語のUI**、右横書き（RTL）対応（アラビア語、ウルドゥー語、パンジャブ語）
- 🖥️ **macOS と Windows** — ユニバーサルMac（Apple Silicon M1〜M4 + Intel）と Windows 10/11（x64）
- 🔒 **あなたのファイルがデバイスの外に出ることはありません** — ファイルのアップロードなし、サーバーなし・送信されるのは匿名かつオプトアウト可能な利用統計のみ・小さなアプリサイズ
- 🆓 **無料・オープンソース**（MIT）

## インストール

### macOS

1. **[最新版の `.dmg` をダウンロード](https://github.com/bandusix/huge-json-viewer/releases/latest)** して開きます。
2. **BigJSON** をアプリケーションフォルダにドラッグします。
3. 初回起動時：このアプリは未署名のため、**アプリを右クリック → 開く** を選び、確認してください（初回のみ）。

動作環境：macOS 11（Big Sur）以降、Apple Silicon または Intel。

### Windows

1. **[最新版の `.exe` をダウンロード](https://github.com/bandusix/huge-json-viewer/releases/latest)**（`BigJSON_x.y.z_x64-setup.exe` の NSIS インストーラー）して実行します。ユーザー単位でインストールされ、管理者権限は不要です。
2. このビルドは未署名のため、**SmartScreen** が表示された場合は **詳細情報 → 実行** をクリックしてください（初回のみ）。

動作環境：Windows 10 または 11（64ビット）。WebView2 は Windows 11 および最新の Windows 10 にプリインストールされていますが、見つからない場合はインストーラーが自動的に取得します。

どちらのビルドも最大4 GBまでのJSONファイルに対応します。

## 仕組み

2〜3 GBのJSONファイルは、メモリ上のオブジェクトとしてパースすることはできません。そこでRust製のコアエンジンは、次のように動作します。

1. ファイルを**メモリマップ**します（`memmap2`）— OSが必要に応じてページ単位で読み込み、ヒープには保持しません。
2. **トークナイザで一度だけストリーミング走査**し、バイトオフセットと構造を表すコンパクトなフラットインデックス（JSONノード1つあたり約23バイト）を構築します — オブジェクトとしてパースはしません。
3. **表示中の行だけを描画します。** ツリーは完全に仮想化されており、展開／折りたたみは文書全体を実体化するのではなく、表示行リストを更新するだけです。**スケーリングされたスクロールバー**により、ブラウザの要素高さ制限を超える数百万行でもスクロールできます。
4. mmap上でSIMDによる部分文字列／正規表現検索を行い、**生バイト列を検索**して、すべてのヒットを対応するノードにマッピングします。

**Tauri v2**（Rustバックエンド + Webフロントエンド）で構築し、約2 MBの `.dmg`（macOS）または NSIS `.exe` インストーラー（Windows）としてパッケージ化しています。

## BigJSON と Dadroit とテキストエディタの比較

事実に基づく、機能ごとの比較です。**BigJSON は無料・オープンソースの [Dadroit](https://dadroit.com) 代替です。** 以下の項目 — 大容量ファイル、検索、CSV/XML エクスポート、複数ファイルの結合、商用利用 — はすべて**無料**で利用できますが、Dadroit ではこれらを **$98〜$198/yr** の有料プランで制限しています。

| | **BigJSON** | **Dadroit** | テキストエディタ（VS Code など） |
| --- | --- | --- | --- |
| **価格** | **無料・オープンソース（MIT）** | 無料 *非商用・≤ 50 MB* · **$98/yr**（≤ 2 GB）· **$198/yr**（≤ 1 TB） | 無料／有料 |
| **商用利用** | ✅ **無料** | 💲 有料（$98/yr〜）— 無料プランは非商用のみ | ✅ |
| **オープンソース** | ✅ MIT、監査可能 | ❌ クローズドソース | 混在 |
| **無料で扱える上限** | **4 GB** | 50 MB、以降は有料 | — |
| **最大ファイルサイズ** | 4 GB／ファイル | **1 TB**（有料プラン） | 数百 MB 程度で限界 |
| **2〜3 GBのJSONを開く** | ✅ 約3秒 | ✅（有料プラン） | ❌ クラッシュ／フリーズ |
| **生の読み込みスループット** | 高速（~1 GB/s、CPU律速） | 非常に高速（ベンダー公称 ~2 GB/s） | 遅い |
| **3 GBファイルのRAM使用量** | ~1.5–2×（インデックス。マップされたファイルは再利用可能なページキャッシュのまま） | ~1×（ベンダー公称） | メモリ不足になることが多い |
| **折りたたみ可能なツリービューア** | ✅ | ✅ | ❌（生テキスト） |
| **キーと値の検索** | ✅ | ✅ | 限定的 |
| **RegEx 検索** | ✅ | ✅ | ✅ |
| **CSV / XML に変換** | ✅ **無料** | ✅ | ❌ |
| **サブツリーを JSON としてエクスポート／コピー** | ✅ | ノード単位のエクスポート | 手動 |
| **キー／値／パスのコピー（jq 形式）** | ✅ | 値 + エクスポート | 手動コピー＆ペースト |
| **複数ファイルの結合** | ✅ **無料** | 💲 有料プラン | ❌ |
| **NDJSON / JSON Lines** | ✅ 自動判定 | ✅ | ❌ |
| **ファイル変更時の自動更新** | ❌ | ✅ | 一部あり |
| **JSON の編集** | ❌ 閲覧のみ | ❌ 閲覧のみ | ✅ |
| **対応プラットフォーム** | macOS（ユニバーサル）· Windows | Windows · macOS · **Linux** | すべて |
| **UI言語** | **20言語（RTL対応）** | 少数 | 多数 |
| **あなたのファイルがデバイスの外に出ることはありません** | ✅ | ✅ | ✅ |
| **インストールサイズ** | ~2–5 MB | 数十 MB | — |

**要するに：** 数 GB までの JSON であれば、BigJSON は Dadroit の有料プランができること — 開く、正規表現検索、CSV/XML への変換、複数ファイルの結合、サブツリーの抽出 — をすべて**無料・オープンソースで、商用利用や機能の課金の壁なしに**こなします。Dadroit の無料プランは **50 MB** で頭打ちになり、商用利用は禁止されています。2 GB／商用利用の解除には **$98/yr**、1 TB + 全機能には **$198/yr** が必要です。

**Dadroit がなお優れている点（正直なところ）：** 4 GB を超えるファイル（最大 **1 TB**）、ネイティブの **Linux** ビルド、ディスク上でファイルが変更されたときの**自動更新**、そして自社ベンチマークでのより高い生スループットです。100 GB〜1 TB のファイルを日常的に開く、あるいは Linux が必要なら、Dadroit はそのライセンス料に見合います。数ギガバイトまでのすべてにおいては、BigJSON が**無料・無制限**の選択肢です。

<sub>Dadroit のプランと価格は [dadroit.com](https://dadroit.com/buy-licence/) に基づきます（変更される場合があります）。速度／RAM の数値は各プロジェクト自身の公称値です。比較は自分のファイルでベンチマークしてください。</sub>

## 対応言語

UIは**20のロケール**で提供され、🌐ボタンから切り替えられます（設定は保存され、初回起動時に自動判定されます）。右横書きのロケールではインターフェースをミラーリングしつつ、JSONツリーは左横書きのまま維持し、数値はロケールに応じてフォーマットされます。

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## ソースからビルド

前提条件：[Node.js](https://nodejs.org) 20以上、[Rustツールチェーン](https://rustup.rs)、およびお使いのOS向けの [Tauri v2 システム前提条件](https://v2.tauri.app/start/prerequisites/)（macOS では Xcode Command Line Tools、Windows では Microsoft C++ Build Tools + WebView2）。

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## 制限事項（v1）

- **ファイルサイズ：** 最大4 GBまで（コンパクトな `u32` オフセットのため）。これより大きいファイルは、わかりやすいメッセージとともに拒否されます。
- **RAM：** インデックスは1ノードあたり約23バイトのため、2〜3 GBのファイルにはおよそ**ファイルサイズの1.5〜2倍**のRAMが必要です（メモリマップされたファイル自体は、OSが破棄可能なページキャッシュです）。16 GBのマシンなら2〜3 GBのファイルを余裕で扱えます。
- **検索**はファイルの生バイト列に対して行われます（エスケープされた文字はエスケープされた形で一致します）。大文字小文字を区別しないマッチングはASCIIのみ対応です。

## ライセンス

[MIT](LICENSE) © bandusix

<sub>キーワード：大きいJSONファイル 開く, 巨大JSON ビューア, 大容量JSONファイル, JSONファイル 開けない, macOS JSONビューア, Windows JSONビューア, ネイティブ Mac JSONビューア, Windows で大きいJSONを開く, 無料 Dadroit 代替, オープンソース JSONビューア, 大きいJSON 検索, JSON サブツリー 抽出, JSON の値／パス コピー, JSON to CSV / XML, NDJSON ビューア, ギガバイト JSON, ストリーミング JSONビューア。</sub>
