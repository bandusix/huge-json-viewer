import { api, onProgress, onExportProgress, type OpenSummary, type RowView, type SearchOpts } from "./ipc";
import { mountIcons, TWISTY_SVG, ICONS } from "./icons";
import {
  t,
  fmtNum,
  initLocale,
  applyI18n,
  setLocale,
  currentLocale,
  LOCALES,
} from "./i18n";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { getCurrentWebview } from "@tauri-apps/api/webview";

// ---- layout constants (keep in sync with styles.css) ----
const ROW_H = 22;
const INDENT = 15;
const OVERSCAN = 16;
// Browsers cap element height (~16.7M–33.5M px). Stay well below and remap
// scroll position for taller virtual content so millions of rows still scroll.
const MAX_SPACER = 15_000_000;

// ---- DOM ----
const $ = <T extends HTMLElement = HTMLElement>(id: string) =>
  document.getElementById(id) as T;

const els = {
  titleBar: $("titlebar-title"),
  btnOpen: $("btn-open"),
  btnOpen2: $("btn-open-2"),
  btnTheme: $("btn-theme"),
  btnCollapse: $("btn-collapse"),
  btnExpand: $("btn-expand"),
  btnExport: $("btn-export"),
  exportMenu: $("export-menu"),
  toast: $("toast"),
  btnCancelExport: $("btn-cancel-export"),
  btnPrev: $("btn-prev"),
  btnNext: $("btn-next"),
  search: $("search"),
  searchInput: $<HTMLInputElement>("search-input"),
  searchCount: $("search-count"),
  tglKeys: $("tgl-keys"),
  tglValues: $("tgl-values"),
  tglCase: $("tgl-case"),
  tglRegex: $("tgl-regex"),
  progress: $("progress"),
  progressBar: $("progress-bar"),
  viewport: $("viewport"),
  empty: $("empty"),
  tree: $("tree"),
  spacer: $("tree-spacer"),
  rows: $("tree-rows"),
  dropOverlay: $("drop-overlay"),
  loading: $("loading"),
  loadingTitle: $("loading-title"),
  loadingSub: $("loading-sub"),
  statusPath: $("status-path"),
  statusNodes: $("status-nodes"),
  statusSize: $("status-size"),
  statusTime: $("status-time"),
};

// ---- utils ----
const fmtInt = (n: number) => fmtNum(n);
function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const u = ["KB", "MB", "GB", "TB"];
  let i = -1;
  do {
    n /= 1024;
    i++;
  } while (n >= 1024 && i < u.length - 1);
  return `${n.toFixed(n >= 100 ? 0 : n >= 10 ? 1 : 2)} ${u[i]}`;
}
function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
function clamp(v: number, lo: number, hi: number) {
  return v < lo ? lo : v > hi ? hi : v;
}
function debounce<T extends (...a: any[]) => void>(fn: T, ms: number) {
  let t: number | undefined;
  return (...a: Parameters<T>) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...a), ms) as unknown as number;
  };
}

// ---- app state ----
let summary: OpenSummary | null = null;
let visibleCount = 0;
let selectedId = -1;
let selectedVi = -1;
let currentMatchNodeId = -1;

const rowCache = new Map<number, RowView>();
const inflight = new Set<string>();

// search
let searchTotal = 0;
let searchCapped = false;
let searchCurrent = -1;
let activeQuery = "";
const searchOpts: SearchOpts = { keys: true, values: true, caseSensitive: false, regex: false };

// ---- theme ----
function initTheme() {
  const saved = localStorage.getItem("theme");
  const theme =
    saved ??
    (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  document.documentElement.dataset.theme = theme;
}
els.btnTheme.addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  document.documentElement.dataset.theme = next;
  localStorage.setItem("theme", next);
});

// ---- search-highlight helpers ----
function buildQueryRegex(): RegExp | null {
  if (!activeQuery) return null;
  const flags = searchOpts.caseSensitive ? "g" : "gi";
  try {
    const pattern = searchOpts.regex
      ? activeQuery
      : activeQuery.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return new RegExp(pattern, flags);
  } catch {
    return null;
  }
}

function highlight(text: string, isKey: boolean): string {
  const enabled = isKey ? searchOpts.keys : searchOpts.values;
  const re = enabled ? buildQueryRegex() : null;
  if (!re) return escapeHtml(text);
  let out = "";
  let last = 0;
  let m: RegExpExecArray | null;
  re.lastIndex = 0;
  while ((m = re.exec(text)) !== null) {
    out += escapeHtml(text.slice(last, m.index));
    out += `<span class="hl">${escapeHtml(m[0])}</span>`;
    last = m.index + m[0].length;
    if (m[0].length === 0) re.lastIndex++; // avoid infinite loop on empty match
  }
  out += escapeHtml(text.slice(last));
  return out;
}

// ---- row rendering ----
function countWord(kind: string, n: number): string {
  const obj = kind === "object";
  if (currentLocale() === "en-US")
    return obj ? (n === 1 ? "key" : "keys") : (n === 1 ? "item" : "items");
  return t(obj ? "unit.keys" : "unit.items");
}

function indentStyle(w: number): string {
  if (w <= 0) return `width:0`;
  return (
    `width:${w}px;` +
    `background-image:repeating-linear-gradient(to right,` +
    `transparent 0,transparent ${INDENT - 1}px,` +
    `var(--guide) ${INDENT - 1}px,var(--guide) ${INDENT}px)`
  );
}

function valueHTML(row: RowView): string {
  if (row.container) {
    const [ob, cb] = row.kind === "object" ? ["{", "}"] : ["[", "]"];
    if (row.childCount === 0) return `<span class="k-punct">${ob}${cb}</span>`;
    const label = `<span class="k-count">${fmtInt(row.childCount)} ${countWord(row.kind, row.childCount)}</span>`;
    if (row.expanded) return `<span class="k-punct">${ob}</span>${label}`;
    return `<span class="k-punct">${ob}</span><span class="k-preview"> … </span><span class="k-punct">${cb}</span>${label}`;
  }
  const v = row.preview ?? "";
  const trunc = row.previewTruncated ? `<span class="k-preview">…</span>` : "";
  switch (row.kind) {
    case "string":
      return `<span class="k-string">"${highlight(v, false)}"</span>${trunc}`;
    case "number":
      return `<span class="k-number">${highlight(v, false)}</span>${trunc}`;
    case "bool":
      return `<span class="k-bool">${escapeHtml(v)}</span>`;
    default:
      return `<span class="k-null">null</span>`;
  }
}

function rowHTML(vi: number, row: RowView | undefined, virtualTop: number): string {
  const top = vi * ROW_H - virtualTop;
  if (!row) {
    return (
      `<div class="row skeleton" style="top:${top}px" data-vi="${vi}">` +
      `<span class="row-gutter">${vi + 1}</span>` +
      `<div class="row-body"><span class="sk"></span></div></div>`
    );
  }
  const cls = ["row"];
  if (row.id === selectedId) cls.push("selected");
  if (row.container && row.expanded) cls.push("expanded");
  if (row.id === currentMatchNodeId) cls.push("match-current");

  const expandable = row.container && row.childCount > 0;
  let body = `<span class="row-indent" style="${indentStyle(row.depth * INDENT)}"></span>`;
  body += `<span class="twisty${expandable ? "" : " leaf"}">${expandable ? TWISTY_SVG : ""}</span>`;
  if (row.key !== null) {
    body += `<span class="k-key">"${highlight(row.key, true)}"</span><span class="k-colon">:</span>`;
  }
  body += valueHTML(row);

  return (
    `<div class="${cls.join(" ")}" style="top:${top}px" data-vi="${vi}" data-id="${row.id}">` +
    `<span class="row-gutter">${row.line}</span>` +
    `<div class="row-body">${body}</div></div>`
  );
}

// ---- virtual scroll ----
let renderQueued = false;
function scheduleRender() {
  if (renderQueued) return;
  renderQueued = true;
  requestAnimationFrame(() => {
    renderQueued = false;
    doRender();
  });
}

function geometry() {
  const viewportH = els.tree.clientHeight;
  const contentH = visibleCount * ROW_H;
  const spacerH = Math.min(contentH, MAX_SPACER);
  const maxScroll = Math.max(0, spacerH - viewportH);
  const maxVirtual = Math.max(0, contentH - viewportH);
  return { viewportH, contentH, spacerH, maxScroll, maxVirtual };
}

function doRender() {
  if (!summary) return;
  const g = geometry();
  els.spacer.style.height = `${g.spacerH}px`;
  const scrollTop = clamp(els.tree.scrollTop, 0, g.maxScroll);
  const virtualTop = g.maxScroll > 0 ? (scrollTop / g.maxScroll) * g.maxVirtual : 0;

  const firstRow = Math.floor(virtualTop / ROW_H);
  const rowsInView = Math.ceil(g.viewportH / ROW_H) + 1;
  const start = Math.max(0, firstRow - OVERSCAN);
  const end = Math.min(visibleCount, firstRow + rowsInView + OVERSCAN);

  ensureRows(start, end);

  els.rows.style.transform = `translateY(${scrollTop}px)`;
  let html = "";
  for (let vi = start; vi < end; vi++) {
    html += rowHTML(vi, rowCache.get(vi), virtualTop);
  }
  els.rows.innerHTML = html;
}

async function ensureRows(start: number, end: number) {
  let missing = false;
  for (let i = start; i < end; i++) {
    if (!rowCache.has(i)) {
      missing = true;
      break;
    }
  }
  if (!missing) return;
  const key = `${start}:${end}`;
  if (inflight.has(key)) return;
  inflight.add(key);
  try {
    const resp = await api.getRows(start, end - start);
    if (resp.visibleCount !== visibleCount) {
      visibleCount = resp.visibleCount;
    }
    resp.rows.forEach((r, idx) => rowCache.set(start + idx, r));
    if (rowCache.size > 6000) {
      for (const k of rowCache.keys()) {
        if (k < start - 2000 || k > end + 2000) rowCache.delete(k);
      }
    }
    scheduleRender();
  } catch (e) {
    console.error("get_rows failed", e);
  } finally {
    inflight.delete(key);
  }
}

function invalidate(newVisibleCount: number) {
  visibleCount = newVisibleCount;
  rowCache.clear();
  inflight.clear();
}

function scrollToRow(vi: number, align: "center" | "top" | "nearest" = "nearest") {
  const g = geometry();
  const curVirtual = g.maxScroll > 0 ? (els.tree.scrollTop / g.maxScroll) * g.maxVirtual : 0;
  let target: number;
  if (align === "center") target = vi * ROW_H - g.viewportH / 2 + ROW_H / 2;
  else if (align === "top") target = vi * ROW_H;
  else {
    const rowTop = vi * ROW_H;
    const rowBot = rowTop + ROW_H;
    if (rowTop < curVirtual) target = rowTop;
    else if (rowBot > curVirtual + g.viewportH) target = rowBot - g.viewportH;
    else return; // already visible
  }
  target = clamp(target, 0, g.maxVirtual);
  els.tree.scrollTop = g.maxVirtual > 0 ? (target / g.maxVirtual) * g.maxScroll : 0;
  scheduleRender();
}

els.tree.addEventListener("scroll", scheduleRender, { passive: true });
window.addEventListener("resize", scheduleRender);

// ---- selection & breadcrumb ----
const updateBreadcrumb = debounce(async (nodeId: number) => {
  try {
    const segs = await api.breadcrumb(nodeId);
    els.statusPath.innerHTML = segs
      .map((s, i) => {
        const sep = i > 0 ? `<span class="sep"> › </span>` : "";
        const label = s.kind === "index" ? `[${escapeHtml(s.label)}]` : escapeHtml(s.label);
        return sep + label;
      })
      .join("");
  } catch {
    /* ignore */
  }
}, 40);

function selectRow(vi: number, id: number) {
  selectedVi = vi;
  selectedId = id;
  els.btnExpand.toggleAttribute("disabled", false);
  updateBreadcrumb(id);
  doRender();
}

// ---- toggle / expand ----
async function toggleAt(vi: number) {
  try {
    const res = await api.toggle(vi);
    invalidate(res.visibleCount);
    scheduleRender();
  } catch (e) {
    console.error(e);
  }
}

async function collapseAll() {
  if (!summary) return;
  try {
    const vc = await api.collapseAll();
    invalidate(vc);
    els.tree.scrollTop = 0;
    scheduleRender();
  } catch (e) {
    console.error(e);
  }
}

// ---- clicks / keyboard ----
els.rows.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  const rowEl = target.closest<HTMLElement>(".row");
  if (!rowEl || rowEl.classList.contains("skeleton")) return;
  const vi = Number(rowEl.dataset.vi);
  const id = Number(rowEl.dataset.id);
  const onTwisty = !!target.closest(".twisty");
  const isContainerRow = !!rowEl.querySelector(".twisty:not(.leaf)");
  selectRow(vi, id);
  if (onTwisty || (isContainerRow && e.detail === 2)) toggleAt(vi);
});

els.viewport.addEventListener("keydown", (e) => {
  if (!summary) return;
  const page = Math.max(1, Math.floor(els.tree.clientHeight / ROW_H) - 1);
  const move = (toVi: number) => {
    const vi = clamp(toVi, 0, visibleCount - 1);
    scrollToRow(vi, "nearest");
    const cached = rowCache.get(vi);
    if (cached) selectRow(vi, cached.id);
    else {
      selectedVi = vi;
      ensureRows(vi, vi + 1).then(() => {
        const r = rowCache.get(vi);
        if (r) selectRow(vi, r.id);
      });
    }
  };
  switch (e.key) {
    case "ArrowDown": e.preventDefault(); move(selectedVi + 1); break;
    case "ArrowUp": e.preventDefault(); move(selectedVi - 1); break;
    case "PageDown": e.preventDefault(); move(selectedVi + page); break;
    case "PageUp": e.preventDefault(); move(selectedVi - page); break;
    case "Home": e.preventDefault(); move(0); break;
    case "End": e.preventDefault(); move(visibleCount - 1); break;
    case "Enter":
    case " ": {
      e.preventDefault();
      const r = rowCache.get(selectedVi);
      if (r && r.container && r.childCount > 0) toggleAt(selectedVi);
      break;
    }
    case "ArrowRight": {
      e.preventDefault();
      const r = rowCache.get(selectedVi);
      if (r && r.container && r.childCount > 0 && !r.expanded) toggleAt(selectedVi);
      break;
    }
    case "ArrowLeft": {
      e.preventDefault();
      const r = rowCache.get(selectedVi);
      if (r && r.container && r.expanded) toggleAt(selectedVi);
      break;
    }
  }
});

// ---- search ----
function setChip(el: HTMLElement, on: boolean) {
  el.dataset.on = String(on);
}
function readChips() {
  searchOpts.keys = els.tglKeys.dataset.on === "true";
  searchOpts.values = els.tglValues.dataset.on === "true";
  searchOpts.caseSensitive = els.tglCase.dataset.on === "true";
  searchOpts.regex = els.tglRegex.dataset.on === "true";
}
for (const el of [els.tglKeys, els.tglValues, els.tglCase, els.tglRegex]) {
  el.addEventListener("click", () => {
    setChip(el, el.dataset.on !== "true");
    readChips();
    runSearch();
  });
}

function updateSearchCount() {
  if (!activeQuery) {
    els.searchCount.textContent = "";
    els.searchCount.classList.remove("no-match");
    els.btnPrev.toggleAttribute("disabled", true);
    els.btnNext.toggleAttribute("disabled", true);
    return;
  }
  if (searchTotal === 0) {
    els.searchCount.textContent = t("search.noResults");
    els.searchCount.classList.add("no-match");
  } else {
    const suffix = searchCapped ? "+" : "";
    els.searchCount.textContent = `${searchCurrent + 1} / ${fmtInt(searchTotal)}${suffix}`;
    els.searchCount.classList.remove("no-match");
  }
  els.btnPrev.toggleAttribute("disabled", searchTotal === 0);
  els.btnNext.toggleAttribute("disabled", searchTotal === 0);
}

async function gotoMatch(i: number) {
  if (searchTotal === 0) return;
  searchCurrent = ((i % searchTotal) + searchTotal) % searchTotal;
  try {
    const rev = await api.revealMatch(searchCurrent);
    invalidate(rev.visibleCount);
    currentMatchNodeId = rev.nodeId;
    selectedId = rev.nodeId;
    selectedVi = rev.visibleIndex;
    scrollToRow(rev.visibleIndex, "center");
    updateBreadcrumb(rev.nodeId);
    updateSearchCount();
    scheduleRender();
  } catch (e) {
    console.error(e);
  }
}

const runSearch = debounce(async () => {
  if (!summary) return;
  activeQuery = els.searchInput.value;
  if (!activeQuery) {
    searchTotal = 0;
    searchCurrent = -1;
    currentMatchNodeId = -1;
    updateSearchCount();
    scheduleRender();
    return;
  }
  try {
    const res = await api.search(activeQuery, searchOpts);
    searchTotal = res.total;
    searchCapped = res.capped;
    searchCurrent = -1;
    if (searchTotal > 0) {
      await gotoMatch(0);
    } else {
      currentMatchNodeId = -1;
      updateSearchCount();
      scheduleRender();
    }
  } catch (e) {
    els.searchCount.textContent = t("search.invalid");
    els.searchCount.classList.add("no-match");
    console.error(e);
  }
}, 220);

els.searchInput.addEventListener("input", runSearch);
els.searchInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    if (searchTotal > 0) gotoMatch(searchCurrent + (e.shiftKey ? -1 : 1));
  } else if (e.key === "Escape") {
    els.searchInput.value = "";
    runSearch();
    els.viewport.focus();
  }
});
els.btnNext.addEventListener("click", () => gotoMatch(searchCurrent + 1));
els.btnPrev.addEventListener("click", () => gotoMatch(searchCurrent - 1));

// ---- open file ----
function showLoading(title: string) {
  els.loading.hidden = false;
  els.loadingTitle.textContent = title;
  els.loadingSub.textContent = "";
  els.btnCancelExport.hidden = true;
  els.progress.hidden = false;
  els.progressBar.style.width = "0%";
}
function hideLoading() {
  els.loading.hidden = true;
  els.progress.hidden = true;
  els.btnCancelExport.hidden = true;
}

function applyOpened(sum: OpenSummary) {
  summary = sum;
  invalidate(sum.visibleCount);
  selectedId = -1;
  selectedVi = -1;
  currentMatchNodeId = -1;
  activeQuery = "";
  els.searchInput.value = "";
  els.searchInput.toggleAttribute("disabled", false);
  els.btnCollapse.toggleAttribute("disabled", false);
  els.btnExport.toggleAttribute("disabled", false);
  els.empty.hidden = true;
  els.tree.hidden = false;
  els.tree.scrollTop = 0;
  els.titleBar.textContent = sum.fileName;
  document.title = `${sum.fileName} — Huge JSON Viewer`;
  updateStatus();
  updateSearchCount();
  scheduleRender();
  els.viewport.focus();
  if (sum.union && sum.union.skipped.length) {
    showToast(t("union.skipped").replace("{n}", String(sum.union.skipped.length)), null, true);
  }
}

async function openPath(path: string) {
  if (exporting) return;
  showLoading(t("loading.title"));
  try {
    applyOpened(await api.openFile(path));
  } catch (e) {
    alert(`${t("error.openFailed")}\n\n${e}`);
  } finally {
    hideLoading();
  }
}

async function openUnionPaths(paths: string[]) {
  if (exporting) return;
  if (paths.length === 1) return openPath(paths[0]);
  showLoading(t("loading.union"));
  try {
    applyOpened(await api.openUnion(paths));
  } catch (e) {
    alert(`${t("error.openFailed")}\n\n${e}`);
  } finally {
    hideLoading();
  }
}

function updateStatus() {
  if (!summary) return;
  const s = summary;
  els.statusNodes.textContent = `${fmtInt(s.nodeCount)} ${t("status.nodes")}`;
  els.statusSize.textContent = fmtBytes(s.fileSize);
  const secs = s.loadMs / 1000;
  const timeStr = s.loadMs < 1000 ? `${s.loadMs} ms` : `${secs.toFixed(secs < 10 ? 2 : 1)} s`;
  els.statusTime.textContent = `${t("status.indexedIn")} ${timeStr}${s.ndjson ? " · NDJSON" : ""}`;
  els.statusPath.textContent = "";
}

async function chooseFile() {
  const picked = await openDialog({
    multiple: true,
    directory: false,
    filters: [{ name: "JSON", extensions: ["json", "ndjson", "jsonl", "txt", "geojson"] }],
  });
  if (Array.isArray(picked)) {
    if (picked.length === 1) openPath(picked[0]);
    else if (picked.length > 1) openUnionPaths(picked);
  } else if (typeof picked === "string") {
    openPath(picked);
  }
}

els.btnOpen.addEventListener("click", chooseFile);
els.btnOpen2.addEventListener("click", chooseFile);
els.btnCollapse.addEventListener("click", collapseAll);
els.btnExpand.addEventListener("click", () => {
  if (selectedVi >= 0) {
    const r = rowCache.get(selectedVi);
    if (r && r.container && r.childCount > 0 && !r.expanded) toggleAt(selectedVi);
  }
});

// ---- global shortcuts ----
window.addEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "o") {
    e.preventDefault();
    chooseFile();
  } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
    e.preventDefault();
    if (summary) {
      els.searchInput.focus();
      els.searchInput.select();
    }
  }
});

// ---- drag & drop (Tauri v2 native); guarded so a missing webview API
//      (e.g. opened in a plain browser) can't abort the rest of boot ----
try {
  getCurrentWebview()
    .onDragDropEvent((event) => {
      const p = event.payload as { type: string; paths?: string[] };
      if (p.type === "over" || p.type === "enter") {
        els.dropOverlay.hidden = false;
      } else if (p.type === "leave") {
        els.dropOverlay.hidden = true;
      } else if (p.type === "drop") {
        els.dropOverlay.hidden = true;
        const paths = p.paths ?? [];
        if (paths.length === 1) openPath(paths[0]);
        else if (paths.length > 1) openUnionPaths(paths);
      }
    })
    .catch((e) => console.error("dragdrop", e));

  // ---- indexing progress ----
  onProgress((p) => {
    if (els.loading.hidden || exporting) return;
    const pct = p.bytesTotal > 0 ? (p.bytesDone / p.bytesTotal) * 100 : 0;
    els.progressBar.style.width = `${Math.min(100, pct).toFixed(1)}%`;
    els.loadingSub.textContent = `${fmtBytes(p.bytesDone)} / ${fmtBytes(p.bytesTotal)} · ${pct.toFixed(0)}%`;
  }).catch((e) => console.error("progress listen", e));

  onExportProgress((p) => {
    if (els.loading.hidden || !exporting) return;
    const pct = p.bytesTotal > 0 ? (p.bytesDone / p.bytesTotal) * 100 : 0;
    els.progressBar.style.width = `${Math.min(100, pct).toFixed(1)}%`;
    els.loadingSub.textContent = `${Math.min(100, pct).toFixed(0)}%`;
  }).catch((e) => console.error("export progress listen", e));
} catch (e) {
  console.warn("Tauri webview APIs unavailable (running outside the app?)", e);
}

// ---- language menu ----
const langMenu = $("lang-menu");
const btnLang = $("btn-lang");

function buildLangMenu() {
  langMenu.innerHTML = LOCALES.map(
    (l) =>
      `<button class="lang-item" data-code="${l.code}" dir="${l.rtl ? "rtl" : "ltr"}">` +
      `<span class="lang-check">${ICONS.check}</span>` +
      `<span class="lang-name">${l.name}</span>` +
      `<span class="lang-code">${l.code}</span></button>`,
  ).join("");
  markActiveLang();
}
function markActiveLang() {
  langMenu.querySelectorAll<HTMLElement>(".lang-item").forEach((el) => {
    el.classList.toggle("active", el.dataset.code === currentLocale());
  });
}
function toggleLangMenu(show?: boolean) {
  const willShow = show ?? langMenu.hidden;
  langMenu.hidden = !willShow;
  if (willShow) markActiveLang();
}
function applyLocale(code: string) {
  setLocale(code);
  applyI18n();
  markActiveLang();
  if (summary) updateStatus();
  updateSearchCount();
  els.loadingTitle.textContent = t("loading.title");
  scheduleRender();
}

btnLang.addEventListener("click", (e) => {
  e.stopPropagation();
  toggleLangMenu();
});
langMenu.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest<HTMLElement>(".lang-item");
  if (item?.dataset.code) {
    applyLocale(item.dataset.code);
    toggleLangMenu(false);
  }
});
document.addEventListener("click", (e) => {
  if (
    !langMenu.hidden &&
    !langMenu.contains(e.target as Node) &&
    !btnLang.contains(e.target as Node)
  ) {
    toggleLangMenu(false);
  }
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !langMenu.hidden) toggleLangMenu(false);
});

// ---- export ----
let exporting = false;
let toastTimer: number | undefined;

function showToast(
  msg: string,
  action: { label: string; fn: () => void } | null,
  isError: boolean,
) {
  clearTimeout(toastTimer);
  els.toast.className = "toast" + (isError ? " error" : "");
  els.toast.textContent = "";
  const m = document.createElement("span");
  m.className = "toast-msg";
  m.textContent = msg;
  els.toast.appendChild(m);
  if (action) {
    const a = document.createElement("span");
    a.className = "toast-action";
    a.textContent = action.label;
    a.addEventListener("click", () => {
      action.fn();
      hideToast();
    });
    els.toast.appendChild(a);
  }
  els.toast.hidden = false;
  toastTimer = setTimeout(hideToast, action ? 9000 : 5000) as unknown as number;
}
function hideToast() {
  els.toast.hidden = true;
}

function buildExportMenu() {
  const mk = (fmt: string, node: number, label: string) =>
    `<button class="menu-item" data-fmt="${fmt}" data-node="${node}">` +
    `<span class="mi-ico">${ICONS.export}</span><span>${escapeHtml(label)}</span></button>`;
  let html = `<div class="menu-label">${escapeHtml(t("export.whole"))}</div>`;
  html += mk("csv", 0, t("export.asCsv"));
  html += mk("xml", 0, t("export.asXml"));
  if (selectedId >= 0) {
    html += `<div class="menu-sep"></div><div class="menu-label">${escapeHtml(t("export.selection"))}</div>`;
    html += mk("csv", selectedId, t("export.asCsv"));
    html += mk("xml", selectedId, t("export.asXml"));
  }
  els.exportMenu.innerHTML = html;
}
function toggleExportMenu(show?: boolean) {
  const willShow = show ?? els.exportMenu.hidden;
  if (willShow) buildExportMenu();
  els.exportMenu.hidden = !willShow;
}

async function runExport(format: "csv" | "xml", nodeId: number) {
  if (!summary || exporting) return;
  const base = summary.fileName.replace(/\.[^.]*$/, "") || "export";
  let dest: string | null;
  try {
    dest = await saveDialog({
      defaultPath: `${base}.${format}`,
      filters: [{ name: format.toUpperCase(), extensions: [format] }],
    });
  } catch {
    return;
  }
  if (!dest) return;
  exporting = true;
  showLoading(t("export.running"));
  els.btnCancelExport.hidden = false;
  els.loadingSub.textContent = "0%";
  try {
    const stats = await api.export({ nodeId, format, dest });
    if (stats.canceled) {
      showToast(t("export.canceled"), null, false);
    } else {
      const msg = t("export.done")
        .replace("{rows}", fmtInt(stats.rows))
        .replace("{cols}", fmtInt(stats.columns))
        .replace("{size}", fmtBytes(stats.bytesWritten));
      const d = dest;
      showToast(msg, { label: t("action.reveal"), fn: () => revealItemInDir(d).catch(() => {}) }, false);
    }
  } catch (e) {
    showToast(`${t("export.failed")} ${e}`, null, true);
  } finally {
    exporting = false;
    hideLoading();
  }
}

els.btnExport.addEventListener("click", (e) => {
  e.stopPropagation();
  if (!summary) return;
  toggleExportMenu();
});
els.exportMenu.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest<HTMLElement>(".menu-item");
  if (!item) return;
  toggleExportMenu(false);
  runExport(item.dataset.fmt as "csv" | "xml", Number(item.dataset.node));
});
document.addEventListener("click", (e) => {
  if (
    !els.exportMenu.hidden &&
    !els.exportMenu.contains(e.target as Node) &&
    !els.btnExport.contains(e.target as Node)
  ) {
    toggleExportMenu(false);
  }
});
els.btnCancelExport.addEventListener("click", () => {
  api.cancelExport().catch(() => {});
});

// ---- boot ----
initTheme();
initLocale();
mountIcons();
applyI18n();
buildLangMenu();
