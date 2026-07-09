# Huge JSON Viewer

> Abre y busca en **archivos JSON muy grandes (2–3 GB y más)** en **macOS** en segundos: una **alternativa gratis y de código abierto a Dadroit**. Cuando tu editor de texto o tu navegador se bloquea con un archivo JSON grande, esto lo abre al instante.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · **Español** · [Português](README.pt-BR.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)

### [⬇️ Descargar el último DMG](https://github.com/bandusix/huge-json-viewer/releases/latest)

![Huge JSON Viewer](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Tema claro</summary>

![Huge JSON Viewer — tema claro](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Por qué existe

La mayoría de los editores de texto y visores JSON en línea **se bloquean o se congelan al abrir un archivo JSON grande**, porque parsean todo el contenido en memoria: un archivo de 2–3 GB se dispara hasta 15–30 GB de RAM. **Huge JSON Viewer** nunca hace eso. Funciona como un visor JSON en streaming: mapea el archivo en memoria (memory‑map), construye un índice compacto en una única pasada y solo renderiza las filas visibles en pantalla. Por eso **abre JSON de gigabytes en segundos** y busca en todo el archivo al instante, sin superar ~1,5–2× el tamaño del archivo en RAM.

Si alguna vez has buscado *«cómo abrir un archivo JSON grande»*, *«archivo JSON demasiado grande para abrir»* o un **visor JSON para Mac** como **alternativa gratis a Dadroit**, esto está hecho justo para eso.

## Características

- ⚡ **Abre JSON de 2–3 GB en ~3 segundos**: mapeo en memoria, índice en streaming y árbol virtualizado
- 🔍 **Busca claves y valores** en JSON grandes: con o sin distinción de mayúsculas y minúsculas, texto plano o **regex**, recuento de coincidencias en vivo, anterior/siguiente con salto a la coincidencia que expande la ruta automáticamente
- 🌳 **Árbol plegable con resaltado de sintaxis**, con números de línea, guías de indentación, colores por tipo y recuento de hijos (al estilo de Dadroit / jsonviewer.app)
- 📄 Abre `.json`, `.ndjson` / `.jsonl` (detectados automáticamente), `.geojson`, `.txt`
- 🖱️ Arrastrar y soltar, ⌘O para abrir, ⌘F para buscar, navegación completa con el teclado
- 🌍 **Interfaz en 20 idiomas**, compatible con la escritura de derecha a izquierda (árabe, urdu, panyabí)
- 🖥️ **Universal**: Apple Silicon (M1–M4) *e* Intel
- 🔒 **100 % sin conexión**: sin subidas, sin servidor, sin telemetría · app de 2 MB
- 🆓 **Gratis y de código abierto** (MIT)

## Instalación

1. **[Descarga el último `.dmg`](https://github.com/bandusix/huge-json-viewer/releases/latest)** y ábrelo.
2. Arrastra **Huge JSON Viewer** a Aplicaciones.
3. Primer inicio: la app no está firmada, así que **haz clic derecho en la app → Abrir** y luego confirma (solo hace falta una vez).

Requisitos: macOS 11 (Big Sur) o posterior. Admite archivos JSON de hasta 4 GB.

## Cómo funciona

Un archivo JSON de 2–3 GB no se puede parsear como objetos en memoria. En su lugar, el núcleo escrito en Rust:

1. **Mapea en memoria** el archivo (`memmap2`): el sistema operativo lo carga por páginas bajo demanda, sin mantenerlo en el heap.
2. **Ejecuta una única pasada de tokenización en streaming** para construir un índice plano y compacto (~23 bytes por nodo JSON) de desplazamientos de bytes y estructura, nunca objetos parseados.
3. **Solo renderiza las filas visibles.** El árbol está totalmente virtualizado; expandir/plegar modifica una lista de filas visibles en lugar de materializar todo el documento. Una **barra de desplazamiento escalada** mantiene millones de filas desplazables más allá del límite de altura de elementos del navegador.
4. **Busca en los bytes en bruto** con subcadenas SIMD / regex sobre el mmap y asocia cada coincidencia con su nodo.

Construido con **Tauri v2** (backend en Rust + frontend web), empaquetado como un `.dmg` de ~2 MB.

## Huge JSON Viewer frente a otras herramientas para JSON grandes

| | Huge JSON Viewer | Dadroit | Editores de texto (VS Code, etc.) |
| --- | --- | --- | --- |
| Precio | **Gratis y de código abierto (MIT)** | Gratis + Pro de pago | Gratis |
| Abre JSON de 2–3 GB | ✅ ~3 s | ✅ | ❌ se bloquea / se congela |
| RAM para un archivo de 3 GB | **~1,5–2×** | baja | a menudo se queda sin memoria |
| Busca claves **y** valores | ✅ regex | ✅ | limitado |
| macOS nativo (Apple Silicon + Intel) | ✅ universal | ✅ | ✅ |
| Idiomas de la interfaz | **20 (compatible con RTL)** | pocos | muchos |
| Sin conexión / sin telemetría | ✅ | ✅ | ✅ |

## Idiomas

La interfaz está disponible en **20 idiomas**, cambiables desde el botón 🌐 (se recuerda tu elección y se detecta automáticamente en el primer inicio). Los idiomas de derecha a izquierda reflejan la interfaz mientras mantienen el árbol JSON de izquierda a derecha; los números se formatean según cada idioma.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Compilar desde el código fuente

```bash
npm install
npm run tauri dev                      # hot-reloading dev app
npm run tauri build -- --bundles dmg   # build the DMG
cd src-tauri && cargo test             # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Limitaciones (v1)

- **Tamaño de archivo:** hasta 4 GB (desplazamientos `u32` compactos). Los archivos más grandes se rechazan con un mensaje claro.
- **RAM:** el índice ocupa ~23 bytes/nodo, así que un archivo de 2–3 GB necesita aproximadamente **1,5–2× el tamaño del archivo** en RAM (el propio archivo mapeado en memoria es caché de páginas del sistema operativo que se puede liberar). Un Mac de 16 GB maneja archivos de 2–3 GB sin problemas.
- **La búsqueda** coincide con los bytes en bruto del archivo (un carácter escapado coincide en su forma escapada); la búsqueda sin distinción de mayúsculas y minúsculas solo funciona con ASCII.

## Licencia

[MIT](LICENSE) © bandusix

<sub>Palabras clave: abrir archivo JSON grande, visor JSON para archivos grandes, abrir JSON de 2GB/3GB, archivo JSON demasiado grande, visor JSON para Mac, alternativa gratis a Dadroit, visor JSON de código abierto, buscar en JSON grande, JSON de gigabytes, visor JSON en streaming.</sub>
