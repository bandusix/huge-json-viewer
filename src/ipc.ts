// Typed wrappers around the Tauri command surface.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Kind = "object" | "array" | "string" | "number" | "bool" | "null";

export interface OpenSummary {
  path: string;
  fileName: string;
  fileSize: number;
  nodeCount: number;
  visibleCount: number;
  rootKind: Kind;
  loadMs: number;
  ndjson: boolean;
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
};

export function onProgress(cb: (p: ProgressEvent) => void): Promise<UnlistenFn> {
  return listen<ProgressEvent>("index-progress", (e) => cb(e.payload));
}
