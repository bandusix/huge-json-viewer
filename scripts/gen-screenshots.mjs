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
  export: svg(`<path d="M12 3v10.5M8.3 6.7 12 3l3.7 3.7"/><path d="M5 13v6a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-6"/>`),
  globe: svg(`<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3c2.6 2.7 2.6 15.3 0 18M12 3c-2.6 2.7-2.6 15.3 0 18"/>`),
};
const TW = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" width="9" height="9"><path d="m9 6 6 6-6 6"/></svg>`;

const ind = (w) =>
  w <= 0
    ? "width:0"
    : `width:${w}px;background-image:repeating-linear-gradient(to right,transparent 0,transparent 14px,var(--guide) 14px,var(--guide) 15px)`;

const S = (s) => `<span class="k-string">"${s}"</span>`;
const N = (s) => `<span class="k-number">${s}</span>`;
const B = (b) => `<span class="k-bool">${b}</span>`;
const NUL = `<span class="k-null">null</span>`;
const P = (s) => `<span class="k-punct">${s}</span>`;
const CNT = (s) => `<span class="k-count">${s}</span>`;
const PV = `<span class="k-preview"> … </span>`;

// An array of records, expanded — a realistic full-window view.
const cities = ["Shanghai", "Berlin", "Tokyo"];
const names = ["Alice #42", "Bob #17", "Carol #88"];
const uuids = ["u-9e3f1a7c", "u-4b7c2d10", "u-1f8a6e33"];
const rows = [{ depth: 0, exp: true, tw: true, key: null, v: P("[") + CNT("6 items") }];
for (let r = 0; r < 3; r++) {
  rows.push({ depth: 1, exp: true, tw: true, key: null, v: P("{") + CNT("9 keys") });
  rows.push({ depth: 2, key: "id", v: N(r + 1) });
  rows.push({ depth: 2, key: "uuid", v: S(uuids[r]) });
  rows.push({ depth: 2, key: "name", v: S(names[r]) });
  rows.push({ depth: 2, key: "age", v: N(34 - r * 5) });
  rows.push({ depth: 2, key: "active", v: B(r % 2 === 0) });
  rows.push({ depth: 2, key: "score", v: N((843.5 - r * 128.4).toFixed(1)) });
  rows.push({ depth: 2, key: "city", v: S(cities[r]), sel: r === 0 });
  rows.push({ depth: 2, exp: true, tw: true, key: "tags", v: P("[") + CNT("2 items") });
  rows.push({ depth: 3, key: null, v: S("prod") });
  rows.push({ depth: 3, key: null, v: S("review") });
  rows.push({ depth: 2, exp: true, tw: true, key: "meta", v: P("{") + CNT("3 keys") });
  rows.push({ depth: 3, key: "created", v: N(1600000042 + r) });
  rows.push({ depth: 3, key: "note", v: r === 1 ? NUL : S("ok") });
  rows.push({ depth: 3, tw: true, key: "nested", v: P("{") + PV + P("}") + CNT("1 key") });
}
rows.forEach((row, i) => (row.line = i + 1));
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
  <header class="titlebar"><div class="titlebar-lights-spacer"></div><div class="titlebar-title">big.json</div><div class="titlebar-actions"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.globe}</span></button><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.theme}</span></button></div></header>
  <div class="toolbar">
    <button class="btn btn-primary"><span class="ico">${ICON.open}</span><span>Open…</span></button>
    <div class="search"><span class="ico ico-search">${ICON.search}</span><input class="search-input" placeholder="Search keys and values…"><div class="search-count"></div>
      <div class="search-toggles"><button class="tgl" data-on="true">Key</button><button class="tgl" data-on="true">Val</button><button class="tgl" data-on="false">Aa</button><button class="tgl mono" data-on="false">.*</button></div>
      <div class="search-nav"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.up}</span></button><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.down}</span></button></div>
    </div>
    <div class="toolbar-right"><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.collapse}</span></button><button class="btn btn-ghost btn-icon"><span class="ico">${ICON.expand}</span></button><button class="btn btn-ghost"><span class="ico">${ICON.export}</span><span>Export</span></button></div>
  </div>
  <main class="viewport"><div class="tree"><div class="tree-spacer" style="height:${rows.length * 22}px"></div><div class="tree-rows">${rowsHTML}</div></div></main>
  <footer class="statusbar"><div class="status-seg status-path">$<span class="sep"> › </span>[0]<span class="sep"> › </span>city</div><div class="status-spacer"></div><div class="status-seg">28,000,001 nodes</div><div class="status-seg">2.7 GB</div><div class="status-seg">indexed in 2.9 s</div></footer>
</div></body></html>`;

for (const theme of ["dark", "light"]) {
  const p = join(outDir, `${theme}.html`);
  writeFileSync(p, page(theme));
  console.log(p);
}
