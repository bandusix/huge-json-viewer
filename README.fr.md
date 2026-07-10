# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> Ouvrez et recherchez dans des **fichiers JSON volumineux (2–3 Go et plus)** sur **macOS et Windows** en quelques secondes — une **alternative gratuite et open source à Dadroit**. Quand votre éditeur de texte ou votre navigateur plante sur un gros fichier JSON, cette visionneuse JSON volumineux l'ouvre instantanément.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · **Français** · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ Télécharger pour macOS ou Windows](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Journal des modifications](CHANGELOG.md)

![BigJSON](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Thème clair</summary>

![BigJSON — thème clair](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Pourquoi ce projet existe

La plupart des éditeurs de texte et des visionneuses JSON en ligne **plantent ou se figent quand on tente d'ouvrir un gros fichier JSON**, parce qu'ils analysent l'intégralité du contenu en mémoire — un fichier de 2–3 Go gonfle jusqu'à 15–30 Go de RAM. **BigJSON** ne fait jamais cela. L'application mappe le fichier en mémoire (memory-map), construit un index compact en une seule passe de streaming, et n'affiche que les lignes visibles à l'écran. Elle **ouvre un JSON de plusieurs gigaoctets en quelques secondes** et recherche dans tout le fichier instantanément, tout en restant sous ~1,5–2× la taille du fichier en RAM.

Si vous avez déjà cherché *« comment ouvrir un gros fichier JSON »*, *« fichier JSON trop gros pour être ouvert »*, ou une **alternative gratuite à Dadroit pour Mac ou Windows**, c'est exactement pour cela que cet outil a été conçu.

## Fonctionnalités

- ⚡ **Ouvre un JSON de 2–3 Go en ~3 secondes** — memory-map, index en streaming, arbre virtualisé
- 🔍 **Recherche dans les clés et les valeurs** — sensible ou non à la casse, texte brut ou **regex**, comptage des correspondances en direct, précédent/suivant avec saut vers la correspondance qui déplie automatiquement le chemin
- 📤 **Convertit JSON → CSV ou XML** — export en streaming qui fonctionne sur des fichiers de plusieurs Go (un immense tableau d'objets devient un tableur)
- 🔗 **Fusion de plusieurs fichiers** — ouvrez plusieurs fichiers JSON à la fois comme un seul arbre combiné et interrogeable
- 🌳 **Arbre repliable avec coloration syntaxique** : numéros de ligne, guides d'indentation, couleurs par type et nombre d'enfants (style Dadroit / jsonviewer.app)
- 📄 Ouvre les fichiers `.json`, `.ndjson` / `.jsonl` (détection automatique), `.geojson`, `.txt`
- 🖱️ Glisser-déposer, **⌘O / Ctrl+O** pour ouvrir, **⌘F / Ctrl+F** pour rechercher, navigation entièrement au clavier
- 🌍 **Interface en 20 langues**, compatible droite-à-gauche (arabe, ourdou, pendjabi)
- 🖥️ **macOS et Windows** — Mac universel (Apple Silicon M1–M4 + Intel) et Windows 10/11 (x64)
- 🔒 **100 % hors ligne** — aucun envoi, aucun serveur, aucune télémétrie · application de 2 Mo
- 🆓 **Gratuite et open source** (MIT)

## Installation

### macOS

1. **[Téléchargez le dernier fichier `.dmg`](https://github.com/bandusix/huge-json-viewer/releases/latest)** et ouvrez-le.
2. Faites glisser **BigJSON** dans Applications.
3. Au premier lancement : l'application n'est pas signée, donc **faites un clic droit sur l'application → Ouvrir**, puis confirmez (nécessaire une seule fois).

Configuration requise : macOS 11 (Big Sur) ou plus récent, Apple Silicon ou Intel.

### Windows

1. **[Téléchargez le dernier fichier `.exe`](https://github.com/bandusix/huge-json-viewer/releases/latest)** (l'installateur NSIS `BigJSON_x.y.z_x64-setup.exe`) et exécutez-le — il s'installe par utilisateur, sans droits d'administrateur.
2. La version n'est pas signée, donc si **SmartScreen** apparaît, cliquez sur **Informations complémentaires → Exécuter quand même** (nécessaire une seule fois).

Configuration requise : Windows 10 ou 11 (64 bits). WebView2 est préinstallé sur Windows 11 et les versions récentes de Windows 10 ; l'installateur le récupère automatiquement s'il est absent.

Les deux versions prennent en charge les fichiers JSON jusqu'à 4 Go.

## Comment ça marche

Un fichier JSON de 2–3 Go ne peut pas être analysé en objets chargés en mémoire. Le cœur en Rust procède plutôt ainsi :

1. **Memory-map** du fichier (`memmap2`) — pagé à la demande par le système d'exploitation, jamais conservé sur le tas.
2. **Une seule passe de streaming du tokenizer** pour construire un index plat compact (~23 octets par nœud JSON) des décalages d'octets et de la structure — jamais des objets analysés.
3. **N'affiche que les lignes visibles.** L'arbre est entièrement virtualisé ; déplier/replier modifie une liste de lignes visibles au lieu de matérialiser tout le document. Une **barre de défilement mise à l'échelle** garde des millions de lignes défilables au-delà de la limite de hauteur d'élément du navigateur.
4. **Recherche dans les octets bruts** avec sous-chaîne SIMD / regex sur le mmap et relie chaque correspondance à son nœud.

Construite avec **Tauri v2** (backend Rust + frontend web), empaquetée en un `.dmg` d'environ 2 Mo (macOS) ou un installateur NSIS `.exe` (Windows).

## BigJSON face aux autres outils pour gros JSON

| | BigJSON | Dadroit | Éditeurs de texte (VS Code, etc.) |
| --- | --- | --- | --- |
| Prix | **Gratuit et open source (MIT)** | Gratuit + Pro payant | Gratuit |
| Ouvre un JSON de 2–3 Go | ✅ ~3 s | ✅ | ❌ plante / se fige |
| RAM pour un fichier de 3 Go | **~1,5–2×** | faible | souvent en rupture de mémoire |
| Recherche dans les clés **et** les valeurs | ✅ regex | ✅ | limitée |
| Convertir JSON → CSV / XML | ✅ streaming | ✅ | ❌ |
| Fusion de plusieurs fichiers | ✅ | ✅ (niveau Advanced) | ❌ |
| Usage commercial | ✅ **gratuit** | 💲 licence payante | ✅ |
| Natif macOS et Windows | ✅ (Mac universel + Win x64) | ✅ | ✅ |
| Langues de l'interface | **20 (compatible RTL)** | peu | beaucoup |
| Hors ligne / sans télémétrie | ✅ | ✅ | ✅ |

## Langues

L'interface est livrée en **20 locales**, commutables depuis le bouton 🌐 (mémorisé, détecté automatiquement au premier lancement). Les locales de droite à gauche reflètent l'interface tout en gardant l'arbre JSON de gauche à droite ; les nombres sont formatés selon la locale.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Compiler depuis les sources

Prérequis : [Node.js](https://nodejs.org) 20+, la [chaîne d'outils Rust](https://rustup.rs) et les [prérequis système de Tauri v2](https://v2.tauri.app/start/prerequisites/) pour votre système d'exploitation (Xcode Command Line Tools sur macOS ; Microsoft C++ Build Tools + WebView2 sur Windows).

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Limites (v1)

- **Taille de fichier :** jusqu'à 4 Go (décalages compacts `u32`). Les fichiers plus volumineux sont refusés avec un message clair.
- **RAM :** l'index occupe ~23 octets/nœud, donc un fichier de 2–3 Go nécessite environ **1,5–2× la taille du fichier** en RAM (le fichier memory-mappé lui-même est un cache de pages du système, évictable). Une machine de 16 Go gère confortablement des fichiers de 2–3 Go.
- **La recherche** porte sur les octets bruts du fichier (un caractère échappé correspond sous sa forme échappée) ; la correspondance insensible à la casse est limitée à l'ASCII.

## Licence

[MIT](LICENSE) © bandusix

<sub>Mots-clés : ouvrir un gros fichier JSON, visionneuse JSON volumineux, ouvrir un JSON de 2 Go/3 Go, fichier JSON trop gros, visionneuse JSON pour Mac, visionneuse JSON pour Windows, ouvrir un gros JSON sur Windows, alternative gratuite à Dadroit, visionneuse JSON open source, rechercher dans un gros JSON, JSON de plusieurs gigaoctets, visionneuse JSON en streaming.</sub>
