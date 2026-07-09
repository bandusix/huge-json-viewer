#!/usr/bin/env node
// Generate self-contained HTML mockups of the UI (dark + light) for the README.
// Writes two HTML files; a headless browser then screenshots them to docs/*.png.
// Usage: node scripts/gen-screenshots.mjs <outDir>
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(join(here, "..", "src", "styles.css"), "utf8");
const outDir = process.argv[2] ?? join(here, "..", "docs", "_mock");
mkdirSync(outDir, { recursive: true });

const svg = (inner, sw = 1.8) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${inner}</svg>`;
const ICON = {
  open: svg(`<path d="M3 7.5A1.5 1.5 0 0 1 4.5 6h4l2 2.2H19.5A1.5 1.5 0 0 1 21 9.7v8.8A1.5 1.5 0 0 1 19.5 20h-15A1.5 1.5 0 0 1 3 18.5z"/>`),
  search: svg(`<circle cx="11" cy="11" r="6.5"/><path d="m20 20-3.6-3.6"/>`),
  up: svg(`<path d="m6 14 6-6 6 6"/>`),
  down: svg(`<path d="m6 10 6 6 6-6"/>`),
  theme: svg(`<circle cx="12" cy="12" r="4.2"/><path d="M12 2.5v2.4M12 19.1v2.4M2.5 12h2.4M19.1 12h2.4M5.2 5.2l1.7 1.7M17.1 17.1l1.7 1.7M18.8 5.2l-1.7 1.7M6.9 17.1l-1.7 1.7"/>`),
  collapse: svg(`<path d="M4 6h16M8 12h12M8 12l-3-2v4z"/><path d="M4 18h16"/>`),
  expand: svg(`<path d="M4 6h16M8 12h12M4 12l3-2v4z"/><path d="M4 18h16"/>`),
};
const TW = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" width="9" height="9"><path d="m9 6 6 6-6 6"/></svg>`;

const ind = (w) =>
  w <= 0
    ? "width:0"
    : `width:${w}px;background-image:repeating-linear-gradient(to right,transparent 0,transparent 14px,var(--guide) 14px,var(--guide) 15px)`;

const S = (s) => `<span class="k-string">"${s}"</span>`;
const N = (s) => `<span class="k-number">${s}</span>`;
const rows = [
  { line: 1, depth: 0, exp: true, tw: true, key: null, v: `<span class="k-punct">{</span><span class="k-count">9 keys</span>` },
  { line: 2, depth: 1, key: "id", v: N(1) },
  { line: 3, depth: 1, key: "uuid", v: S("u-9e3f1a7c") },
  { line: 4, depth: 1, key: "name", v: S("Alice #42") },
  { line: 5, depth: 1, key: "age", v: N(34) },
  { line: 6, depth: 1, key: "active", v: `<span class="k-bool">true</span>` },
  { line: 7, depth: 1, key: "score", v: N("843.5") },
  { line: 8, depth: 1, key: "city", v: S("Shanghai") },
  { line: 9, depth: 1, exp: true, tw: true, key: "tags", v: `<span class="k-punct">[</span><span class="k-count">2 items</span>` },
  { line: 10, depth: 2, sel: true, key: null, v: S("prod") },
  { line: 11, depth: 2, key: null, v: S("review") },
  { line: 12, depth: 1, exp: true, tw: true, key: "meta", v: `<span class="k-punct">{</span><span class="k-count">3 keys</span>` },
  { line: 13, depth: 2, key: "created", v: N("1600000042") },
  { line: 14, depth: 2, key: "note", v: `<span class="k-null">null</span>` },
  { line: 15, depth: 2, tw: true, key: "nested", v: `<span class="k-punct">{</span><span class="k-preview"> … </span><span class="k-punct">}</span><span class="k-count">1 key</span>` },
];
const rowsHTML = rows
  .map((r, i) => {
    const cls = ["row"];
    if (r.sel) cls.push("selected");
    if (r.exp) cls.push("expanded");
    let body = `<span class="row-indent" style="${ind(r.depth * 15)}"></span>`;
    body += `<span class="twisty${r.tw ? "" : " leaf"}">${r.tw ? TW : ""}</span>`;
    if (r.key != null) body += `<span class="k-key">"${r.key}"</span><span class="k-colon">:</span>`;
    body += r.v;
    return `<div class="${cls.join(" ")}" style="top:${i * 22}px"><span class="row-gutter">${r.line}</span><div class="row-body">${body}</div></div>`;
  })
  .join("");

const page = (theme) => `<!doctype html><html data-theme="${theme}"><head><meta charset="utf-8"><style>
${css}
/* screenshot: draw the traffic-light dots since there is no native title bar */
.titlebar-lights-spacer{position:relative}
.titlebar-lights-spacer::before{content:"";position:absolute;top:50%;left:20px;transform:translateY(-50%);width:12px;height:12px;border-radius:50%;background:#ff5f57;box-shadow:20px 0 #febc2e,40px 0 #28c840}
</style></head><body><div id="app">
  <header class="titlebar"><div class="titlebar-lights-spacer"></div><div class="titlebar-title">big.json</div><div class="titlebar-actions"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.theme}</span></button></div></header>
  <div class="toolbar">
    <button class="btn btn-primary"><span class="ico">${ICON.open}</span><span>Open…</span></button>
    <div class="search"><span class="ico ico-search">${ICON.search}</span><input class="search-input" placeholder="Search keys and values…"><div class="search-count"></div>
      <div class="search-toggles"><button class="tgl" data-on="true">Key</button><button class="tgl" data-on="true">Val</button><button class="tgl" data-on="false">Aa</button><button class="tgl mono" data-on="false">.*</button></div>
      <div class="search-nav"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.up}</span></button><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.down}</span></button></div>
    </div>
    <div class="toolbar-right"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.collapse}</span></button><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.expand}</span></button></div>
  </div>
  <main class="viewport"><div class="tree"><div class="tree-spacer" style="height:${rows.length * 22}px"></div><div class="tree-rows">${rowsHTML}</div></div></main>
  <footer class="statusbar"><div class="status-seg status-path">$<span class="sep"> › </span>tags<span class="sep"> › </span>[0]</div><div class="status-spacer"></div><div class="status-seg">28,000,001 nodes</div><div class="status-seg">2.7 GB</div><div class="status-seg">indexed in 2.9 s</div></footer>
</div></body></html>`;

for (const theme of ["dark", "light"]) {
  const p = join(outDir, `${theme}.html`);
  writeFileSync(p, page(theme));
  console.log(p);
}
