// Anonymous, opt-out usage analytics via the GA4 Measurement Protocol.
//
// PRIVACY CONTRACT — this module must NEVER send: file names, file paths, search
// queries, JSON keys/values, or any file content. Only anonymous event names and
// coarse, non-identifying parameters (platform, app version, a size *bucket*,
// booleans). It is a complete no-op unless BOTH build-time keys are present AND
// the user has not opted out.
//
// Keys are injected at build time from env vars (never committed):
//   VITE_GA_MEASUREMENT_ID = G-XXXXXXXXXX
//   VITE_GA_API_SECRET     = <Measurement Protocol API secret>
import { getVersion } from "@tauri-apps/api/app";

const env = ((import.meta as unknown as { env?: Record<string, string> }).env) ?? {};
const MEASUREMENT_ID = env.VITE_GA_MEASUREMENT_ID;
const API_SECRET = env.VITE_GA_API_SECRET;
const CONFIGURED = !!MEASUREMENT_ID && !!API_SECRET;

const CLIENT_KEY = "ga_client_id";
const OPTOUT_KEY = "analytics"; // "off" ⇒ opted out
const NOTICED_KEY = "analyticsNoticed";

// A per-run session id (GA4 needs session_id + engagement_time_msec to attribute
// events). Date.now() is available in the webview.
const SESSION_ID = String(Date.now());

function ls(): Storage | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

export function analyticsConfigured(): boolean {
  return CONFIGURED;
}
export function optedOut(): boolean {
  return ls()?.getItem(OPTOUT_KEY) === "off";
}
export function analyticsEnabled(): boolean {
  return CONFIGURED && !optedOut();
}
export function setOptOut(off: boolean): void {
  ls()?.setItem(OPTOUT_KEY, off ? "off" : "on");
}
export function noticeSeen(): boolean {
  return ls()?.getItem(NOTICED_KEY) === "1";
}
export function markNoticeSeen(): void {
  ls()?.setItem(NOTICED_KEY, "1");
}

function clientId(): string {
  const store = ls();
  try {
    let id = store?.getItem(CLIENT_KEY) ?? null;
    if (!id) {
      id = crypto.randomUUID();
      store?.setItem(CLIENT_KEY, id);
    }
    return id;
  } catch {
    return "anon";
  }
}

let appVersion = "";
getVersion()
  .then((v) => {
    appVersion = v;
  })
  .catch(() => {});

/** Coarse size bucket — never the exact file size. */
export function sizeBucket(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  if (mb < 1) return "<1MB";
  if (mb < 10) return "1-10MB";
  if (mb < 100) return "10-100MB";
  if (mb < 500) return "100-500MB";
  if (mb < 1024) return "500MB-1GB";
  if (mb < 2048) return "1-2GB";
  return "2GB+";
}

/**
 * Send one anonymous event (fire-and-forget). Silently does nothing when not
 * configured, opted out, or offline. Callers must only pass non-identifying
 * params — never file names, paths, queries, or content.
 */
export function track(name: string, params: Record<string, string | number | boolean> = {}): void {
  if (!analyticsEnabled()) return;
  const platform = document.documentElement.dataset.platform ?? "unknown";
  const body = {
    client_id: clientId(),
    events: [
      {
        name,
        params: {
          ...params,
          platform,
          app_version: appVersion,
          session_id: SESSION_ID,
          engagement_time_msec: 1,
        },
      },
    ],
  };
  const url =
    `https://www.google-analytics.com/mp/collect` +
    `?measurement_id=${encodeURIComponent(MEASUREMENT_ID as string)}` +
    `&api_secret=${encodeURIComponent(API_SECRET as string)}`;
  try {
    fetch(url, { method: "POST", body: JSON.stringify(body), keepalive: true }).catch(() => {});
  } catch {
    /* offline or blocked — ignore */
  }
}
