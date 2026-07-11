# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> 在 **macOS 和 Windows** 上几秒钟打开并搜索**超大 JSON 文件（2–3 GB 甚至更大）**——一款**免费、开源的 Dadroit 替代方案**。当文本编辑器或浏览器一打开大 JSON 文件就崩溃时，它能瞬间打开。

[English](README.md) · **简体中文** · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ 下载 macOS 或 Windows 版](https://github.com/bandusix/huge-json-viewer/releases/latest) · [更新日志](CHANGELOG.md)

![BigJSON——大 JSON 文件查看器](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>浅色主题</summary>

![BigJSON——浅色主题](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## 为什么要做这款工具

大多数文本编辑器和在线 JSON 查看器**一打开大 JSON 文件就崩溃或卡死**，因为它们会把整个文件解析进内存——一个 2–3 GB 的文件会膨胀到 15–30 GB 内存。**BigJSON** 绝不这样做。它把文件做内存映射，用一次流式扫描建立紧凑索引，只渲染屏幕上可见的行。所以它能**几秒钟打开数 GB 级 JSON**，并瞬间搜索整个文件，内存占用始终控制在文件大小的 ~1.5–2 倍以内。

如果你曾搜过 *"怎么打开大 JSON 文件"*、*"JSON 文件太大打不开怎么办"*，或者在找 **Mac 或 Windows 上免费的 Dadroit 替代品**，那么它正是为你而做。

## 功能特性

- ⚡ **约 3 秒打开 2–3 GB JSON**——内存映射、流式索引、虚拟化树
- 🔍 **搜索键和值**——区分或不区分大小写、纯文本或**正则表达式**、实时匹配计数、上一处/下一处跳转并自动展开路径
- 📤 **JSON → CSV 或 XML 转换**——流式导出，可处理数 GB 级文件（把庞大的对象数组变成一张表格）
- 📋 **复制与提取**——右键点击任意节点即可**复制它的键、值、路径**（jq 风格，例如 `.users[3].name`），或**将子树复制 / 导出为 JSON**
- 🔗 **合并多个文件**——一次打开多个 JSON 文件，合成一棵可搜索的组合树
- 🌳 **语法高亮、可折叠的树**，带行号、缩进参考线、类型配色和子节点计数（Dadroit / jsonviewer.app 风格）
- 📄 支持打开 `.json`、`.ndjson` / `.jsonl`（自动识别）、`.geojson`、`.txt`——或直接**从剪贴板粘贴 JSON**
- 🖱️ 拖放打开、**⌘O / Ctrl+O** 打开、**⌘V / Ctrl+V** 粘贴、**⌘F / Ctrl+F** 搜索，全键盘导航
- 🔔 **更新提示**——有新版本发布时，状态栏会出现一个不打扰的链接（每天最多检查一次；完全离线安全）
- 🌍 **20 种语言界面**，支持从右到左布局（阿拉伯语、乌尔都语、旁遮普语）
- 🖥️ **macOS 与 Windows**——通用 Mac（Apple Silicon M1–M4 + Intel）以及 Windows 10/11（x64）
- 🔒 **100% 离线**——不上传、无服务器、无遥测 · 应用仅 2 MB
- 🆓 **免费且开源**（MIT）

## 安装

### macOS

1. **[下载最新的 `.dmg`](https://github.com/bandusix/huge-json-viewer/releases/latest)** 并打开它。
2. 把 **BigJSON** 拖入"应用程序"文件夹。
3. 首次启动：应用未签名，因此请**右键点击应用 → 打开**，然后确认（只需操作一次）。

系统要求：macOS 11（Big Sur）或更高版本，Apple Silicon 或 Intel。

### Windows

1. **[下载最新的 `.exe`](https://github.com/bandusix/huge-json-viewer/releases/latest)**（`BigJSON_x.y.z_x64-setup.exe` NSIS 安装程序）并运行——它按用户安装，无需管理员权限。
2. 该构建未签名，因此如果出现 **SmartScreen** 提示，请点击**更多信息 → 仍要运行**（只需操作一次）。

系统要求：Windows 10 或 11（64 位）。Windows 11 和当前版本的 Windows 10 已预装 WebView2；如缺失，安装程序会自动获取。

两个版本都可处理最大 4 GB 的 JSON 文件。

## 工作原理

2–3 GB 的 JSON 文件无法被解析成内存中的对象。为此，Rust 核心采取的做法是：

1. **内存映射**文件（`memmap2`）——由操作系统按需分页载入，而不是常驻堆内存。
2. **一次流式分词扫描**即建立紧凑的扁平索引（每个 JSON 节点约 23 字节），只记录字节偏移和结构——从不生成解析后的对象。
3. **只渲染可见的行。** 整棵树完全虚拟化；展开/折叠只是改动可见行列表，而非把整个文档实体化。**按比例缩放的滚动条**让数百万行依然可以顺畅滚动，突破浏览器对元素高度的上限。
4. **直接在原始字节上搜索**——在内存映射上用 SIMD 做子串/正则匹配，并把每一处命中映射回对应节点。

采用 **Tauri v2**（Rust 后端 + Web 前端）构建，打包为约 2 MB 的 `.dmg`（macOS）或 NSIS `.exe` 安装程序（Windows）。

## BigJSON 与其他大 JSON 工具对比

| | BigJSON | Dadroit | 文本编辑器（VS Code 等） |
| --- | --- | --- | --- |
| 价格 | **免费且开源（MIT）** | 免费 + 付费 Pro | 免费 |
| 打开 2–3 GB JSON | ✅ 约 3 秒 | ✅ | ❌ 崩溃 / 卡死 |
| 打开 3 GB 文件的内存占用 | **~1.5–2 倍** | 低 | 常常内存溢出 |
| 同时搜索键**和**值 | ✅ 支持正则 | ✅ | 有限 |
| JSON → CSV / XML 转换 | ✅ 流式 | ✅ | ❌ |
| 复制 / 提取值、路径、子树 | ✅ | ✅ | 手动 |
| 合并多个文件 | ✅ | ✅（高级版） | ❌ |
| 商业使用 | ✅ **免费** | 💲 付费授权 | ✅ |
| 原生 macOS 与 Windows | ✅（通用 Mac + Win x64） | ✅ | ✅ |
| 界面语言 | **20 种（支持 RTL）** | 少 | 多 |
| 离线 / 无遥测 | ✅ | ✅ | ✅ |

## 语言

界面共提供 **20 种语言**，可通过 🌐 按钮切换（会记住选择，首次启动时自动识别）。从右到左的语言会镜像整个界面，同时让 JSON 树保持从左到右；数字按各语言习惯格式化。

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## 从源码构建

前置条件：[Node.js](https://nodejs.org) 20+、[Rust 工具链](https://rustup.rs)，以及适用于你操作系统的 [Tauri v2 系统前置条件](https://v2.tauri.app/start/prerequisites/)（macOS 上需 Xcode Command Line Tools；Windows 上需 Microsoft C++ Build Tools + WebView2）。

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## 已知限制（v1）

- **文件大小：** 最大支持 4 GB（使用紧凑的 `u32` 偏移量）。更大的文件会被拒绝并给出明确提示。
- **内存：** 索引每个节点约 23 字节，因此 2–3 GB 的文件大致需要**文件大小的 1.5–2 倍**内存（内存映射的文件本身属于可被系统回收的页缓存）。16 GB 内存的机器可以从容处理 2–3 GB 的文件。
- **搜索**针对文件的原始字节进行匹配（转义字符会以其转义后的形式被匹配）；不区分大小写的匹配仅限 ASCII。

## 许可证

[MIT](LICENSE) © bandusix

<sub>关键词：大JSON文件查看器, 打开大JSON文件, 打开2GB/3GB JSON, JSON文件太大打不开, Mac JSON查看器, Windows JSON查看器, 原生 Mac JSON查看器, 在 Windows 上打开大JSON, 免费 Dadroit 替代, 开源JSON查看器, 搜索大JSON键和值, 提取JSON子树, 复制JSON值 / 路径, JSON转CSV / XML, NDJSON查看器, 千兆JSON, 流式JSON查看器。</sub>
