# BigJSON

**Huge-size JSON Viewer GUI** · _formerly Huge JSON Viewer_

> Abra e pesquise **arquivos JSON muito grandes (2–3 GB ou mais)** no **macOS e no Windows** em segundos — uma **alternativa gratuita e open source ao Dadroit**. Quando seu editor de texto ou navegador trava ao abrir um arquivo JSON grande, este visualizador de JSON grande o abre na hora.

[English](README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · **Português** · [Deutsch](README.de.md) · [Français](README.fr.md) · [Русский](README.ru.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [Türkçe](README.tr.md) · [Bahasa Indonesia](README.id.md)

[![Release](https://img.shields.io/github/v/release/bandusix/huge-json-viewer?color=0a6cff)](https://github.com/bandusix/huge-json-viewer/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/bandusix/huge-json-viewer/total?color=28c840)](https://github.com/bandusix/huge-json-viewer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-black?logo=apple)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-0a6cff?logo=windows)

### [⬇️ Baixe para macOS ou Windows](https://github.com/bandusix/huge-json-viewer/releases/latest) · [Changelog](CHANGELOG.md)

![BigJSON](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-dark.png)

<details><summary>Tema claro</summary>

![BigJSON — tema claro](https://cdn.jsdelivr.net/gh/bandusix/huge-json-viewer@main/docs/screenshot-light.png)

</details>

## Por que este projeto existe

A maioria dos editores de texto e visualizadores de JSON online **trava ou congela quando você abre um arquivo JSON grande**, porque faz o parsing de tudo na memória — um arquivo de 2–3 GB infla para 15–30 GB de RAM. O **BigJSON** nunca faz isso. Ele mapeia o arquivo na memória (memory‑map), constrói um índice compacto em uma única passagem em streaming e renderiza apenas as linhas visíveis na tela. Assim ele **abre JSON de vários gigabytes em segundos** e pesquisa o arquivo inteiro instantaneamente, mantendo o uso de RAM em torno de ~1,5–2× o tamanho do arquivo.

Se você já pesquisou por *"como abrir um arquivo JSON grande"*, *"arquivo JSON muito grande para abrir"* ou uma **alternativa gratuita ao Dadroit para Mac ou Windows**, este visualizador foi feito exatamente para isso.

## Recursos

- ⚡ **Abre JSON de 2–3 GB em ~3 segundos** — memory‑mapped, índice em streaming, árvore virtualizada
- 🔍 **Pesquisa chaves e valores** — com ou sem diferenciação de maiúsculas/minúsculas, texto simples ou **regex**, contagem de resultados ao vivo, próximo/anterior com salto até a ocorrência que expande o caminho automaticamente
- 📤 **Converte JSON → CSV ou XML** — exportação em streaming que funciona em arquivos de vários GB (um array enorme de objetos vira uma planilha)
- 🔗 **Une vários arquivos** — abra vários arquivos JSON de uma vez como uma única árvore combinada e pesquisável
- 🌳 **Árvore recolhível com destaque de sintaxe**, com números de linha, guias de indentação, cores por tipo e contagem de filhos (no estilo Dadroit / jsonviewer.app)
- 📄 Abre `.json`, `.ndjson` / `.jsonl` (detecção automática), `.geojson`, `.txt`
- 🖱️ Arrastar e soltar, **⌘O / Ctrl+O** para abrir, **⌘F / Ctrl+F** para pesquisar, navegação completa pelo teclado
- 🌍 **Interface em 20 idiomas**, compatível com escrita da direita para a esquerda (árabe, urdu, punjabi)
- 🖥️ **macOS e Windows** — Mac universal (Apple Silicon M1–M4 + Intel) e Windows 10/11 (x64)
- 🔒 **100% offline** — sem upload, sem servidor, sem telemetria · app de 2 MB
- 🆓 **Gratuito e open source** (MIT)

## Instalação

### macOS

1. **[Baixe o `.dmg` mais recente](https://github.com/bandusix/huge-json-viewer/releases/latest)** e abra-o.
2. Arraste o **BigJSON** para a pasta Aplicativos.
3. Primeira execução: o app não é assinado, então **clique com o botão direito no app → Abrir** e confirme (necessário apenas uma vez).

Requisitos: macOS 11 (Big Sur) ou mais recente, Apple Silicon ou Intel.

### Windows

1. **[Baixe o `.exe` mais recente](https://github.com/bandusix/huge-json-viewer/releases/latest)** (o instalador NSIS `BigJSON_x.y.z_x64-setup.exe`) e execute-o — a instalação é por usuário, sem necessidade de administrador.
2. O build não é assinado, então se o **SmartScreen** aparecer, clique em **Mais informações → Executar assim mesmo** (necessário apenas uma vez).

Requisitos: Windows 10 ou 11 (64 bits). O WebView2 já vem pré-instalado no Windows 11 e nas versões atuais do Windows 10; o instalador o baixa automaticamente se estiver faltando.

Ambos os builds lidam com arquivos JSON de até 4 GB.

## Como funciona

Um arquivo JSON de 2–3 GB não pode ter seu parsing feito em objetos na memória. Em vez disso, o núcleo em Rust:

1. **Mapeia o arquivo na memória** (`memmap2`) — paginado sob demanda pelo sistema operacional, não mantido na heap.
2. **Faz uma única passagem em streaming com o tokenizer** para construir um índice plano e compacto (~23 bytes por nó JSON) de deslocamentos de bytes e estrutura — nunca objetos com parsing completo.
3. **Renderiza apenas as linhas visíveis.** A árvore é totalmente virtualizada; expandir/recolher altera uma lista de linhas visíveis em vez de materializar o documento inteiro. Uma **barra de rolagem escalonada** mantém milhões de linhas roláveis mesmo além do limite de altura de elementos do navegador.
4. **Pesquisa bytes brutos** com substring SIMD / regex sobre o mmap e mapeia cada ocorrência de volta ao seu nó.

Construído com **Tauri v2** (backend em Rust + frontend web), empacotado como um `.dmg` de ~2 MB (macOS) ou um instalador NSIS `.exe` (Windows).

## BigJSON vs. outras ferramentas para JSON grande

| | BigJSON | Dadroit | Editores de texto (VS Code etc.) |
| --- | --- | --- | --- |
| Preço | **Gratuito e open source (MIT)** | Grátis + Pro US$ 98–198/ano | Grátis |
| Abre JSON de 2–3 GB | ✅ ~3 s | ✅ (2 GB no Standard) | ❌ trava / congela |
| RAM para um arquivo de 3 GB | **~1,5–2×** | baixo | frequentemente fica sem memória |
| Pesquisa chaves **e** valores | ✅ regex | ✅ | limitado |
| Converte JSON → CSV / XML | ✅ streaming | ✅ | ❌ |
| Une vários arquivos | ✅ | ✅ (nível Advanced) | ❌ |
| Uso comercial | ✅ **gratuito** | 💲 licença paga | ✅ |
| macOS e Windows nativos | ✅ (Mac universal + Win x64) | ✅ | ✅ |
| Idiomas da interface | **20 (com suporte a RTL)** | poucos | muitos |
| Offline / sem telemetria | ✅ | ✅ | ✅ |

## Idiomas

A interface é distribuída em **20 idiomas**, alternáveis pelo botão 🌐 (a escolha é salva e detectada automaticamente na primeira execução). Os idiomas escritos da direita para a esquerda espelham a interface, mantendo a árvore JSON da esquerda para a direita; os números seguem o formato de cada idioma.

`en-US` · `zh-CN` · `hi-IN` · `es-ES` · `fr-FR` · `ar-EG` · `bn-BD` · `ru-RU` · `pt-BR` · `id-ID` · `ur-PK` · `de-DE` · `ja-JP` · `sw-TZ` · `mr-IN` · `te-IN` · `pa-PK` · `zh-WUU` · `ta-IN` · `tr-TR`

## Compilar a partir do código-fonte

Pré-requisitos: [Node.js](https://nodejs.org) 20+, o [toolchain do Rust](https://rustup.rs) e os [pré-requisitos de sistema do Tauri v2](https://v2.tauri.app/start/prerequisites/) para o seu SO (Xcode Command Line Tools no macOS; Microsoft C++ Build Tools + WebView2 no Windows).

```bash
npm install
npm run tauri dev                                       # hot-reloading dev app
npm run tauri build -- --target universal-apple-darwin --bundles dmg   # macOS universal DMG
npm run tauri build -- --bundles nsis                   # Windows installer (run on Windows)
cd src-tauri && cargo test                              # engine tests (serde_json oracle, escapes, NDJSON, search)
```

## Limitações (v1)

- **Tamanho do arquivo:** até 4 GB (deslocamentos compactos em `u32`). Arquivos maiores são recusados com uma mensagem clara.
- **RAM:** o índice ocupa ~23 bytes/nó, então um arquivo de 2–3 GB precisa de aproximadamente **1,5–2× o tamanho do arquivo** em RAM (o próprio arquivo mapeado na memória é cache de páginas do SO, que pode ser descartado). Uma máquina com 16 GB lida com arquivos de 2–3 GB com folga.
- **A pesquisa** corresponde aos bytes brutos do arquivo (um caractere escapado casa na sua forma escapada); a correspondência sem diferenciar maiúsculas/minúsculas funciona apenas para ASCII.

## Licença

[MIT](LICENSE) © bandusix

<sub>Palavras-chave: abrir arquivo JSON grande, visualizador de JSON grande, abrir JSON de 2GB/3GB, arquivo JSON muito grande, visualizador JSON para Mac, visualizador JSON para Windows, abrir JSON grande no Windows, alternativa gratuita ao Dadroit, visualizador JSON open source, pesquisar em JSON grande, JSON de gigabytes, visualizador JSON em streaming.</sub>
