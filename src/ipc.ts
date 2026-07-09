// Typed wrappers around the Tauri command surface.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Kind = "object" | "array" | "string" | "number" | "bool" | "null";

export interface SkippedFile {
  name: string;
  error: string;
}
export interface UnionInfo {
  fileCount: number;
  skipped: SkippedFile[];
}

export interface OpenSummary {
  path: string;
  fileName: string;
  fileSize: number;
  nodeCount: number;
  visibleCount: number;
  rootKind: Kind;
  loadMs: number;
  ndjson: boolean;
  union?: UnionInfo | null;
}

export interface CsvOptions {
  delimiter?: "comma" | "semicolon" | "tab";
  bom?: boolean;
  crlf?: boolean;
  nullAsEmpty?: boolean;
  sanitizeFormulas?: boolean;
  maxColumns?: number;
  cellCap?: number;
  nestedAsJson?: boolean;
}
export interface XmlOptions {
  pretty?: boolean;
  declaration?: boolean;
  rootName?: string;
  itemName?: string;
  cellCap?: number;
  preserveKeysAttr?: boolean;
}
export interface ExportRequest {
  nodeId: number;
  format: "csv" | "xml";
  dest: string;
  csv?: CsvOptions;
  xml?: XmlOptions;
}
export interface ExportStats {
  rows: number;
  columns: number;
  bytesWritten: number;
  cellsTruncated: number;
  canceled: boolean;
}
export interface ExportProgress {
  bytesDone: number;
  bytesTotal: number;
  rows: number;
}

export interface RowView {
  id: number;
  line: number;
  depth: number;
  kind: Kind;
  key: string | null;
  keyTruncated: boolean;
  container: boolean;
  expanded: boolean;
  childCount: number;
  preview: string | null;
  previewTruncated: boolean;
}

export interface RowsResponse {
  rows: RowView[];
  visibleCount: number;
}

export interface ToggleResult {
  added: number;
  removed: number;
  visibleCount: number;
  expanded: boolean;
}

export interface PathSeg {
  kind: "root" | "key" | "index";
  label: string;
}

export interface SearchResult {
  total: number;
  capped: boolean;
  queryMs: number;
}

export interface RevealResult {
  nodeId: number;
  visibleIndex: number;
  visibleCount: number;
  isKey: boolean;
}

export interface ProgressEvent {
  bytesDone: number;
  bytesTotal: number;
  nodes: number;
}

export interface SearchOpts {
  keys: boolean;
  values: boolean;
  caseSensitive: boolean;
  regex: boolean;
}

export const api = {
  openFile: (path: string) => invoke<OpenSummary>("open_file", { path }),
  closeFile: () => invoke<void>("close_file"),
  getRows: (start: number, count: number) =>
    invoke<RowsResponse>("get_rows", { start, count }),
  toggle: (visIndex: number) => invoke<ToggleResult>("toggle", { visIndex }),
  collapseAll: () => invoke<number>("collapse_all"),
  breadcrumb: (nodeId: number) => invoke<PathSeg[]>("breadcrumb", { nodeId }),
  revealNode: (nodeId: number) => invoke<RevealResult>("reveal_node", { nodeId }),
  revealMatch: (index: number) => invoke<RevealResult>("reveal_match", { index }),
  search: (query: string, o: SearchOpts) =>
    invoke<SearchResult>("search", {
      query,
      keys: o.keys,
      values: o.values,
      caseSensitive: o.caseSensitive,
      regex: o.regex,
    }),
  openUnion: (paths: string[]) => invoke<OpenSummary>("open_union", { paths }),
  export: (req: ExportRequest) => invoke<ExportStats>("export", { req }),
  cancelExport: () => invoke<void>("cancel_export"),
};

export function onProgress(cb: (p: ProgressEvent) => void): Promise<UnlistenFn> {
  return listen<ProgressEvent>("index-progress", (e) => cb(e.payload));
}

export function onExportProgress(cb: (p: ExportProgress) => void): Promise<UnlistenFn> {
  return listen<ExportProgress>("export-progress", (e) => cb(e.payload));
}
