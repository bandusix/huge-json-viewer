#!/usr/bin/env node
// Pre-render one static HTML page per language for SEO (separate URLs +
// hreflang + self-canonical + localized <title>/description/JSON-LD), from
// scripts/site-template.html and docs/i18n/<code>.json. Also writes sitemap.xml.
//
//   npm i -D cheerio  (once), then:  node scripts/build-site.mjs
import { load } from "cheerio";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const DOCS = path.join(here, "..", "docs");
const BASE = "https://bandusix.github.io/huge-json-viewer/";
const LASTMOD = process.argv[2] || "2026-07-10";

const LOCALES = [
  { code: "en", name: "English", og: "en_US" },
  { code: "zh-CN", name: "简体中文", og: "zh_CN" },
  { code: "zh-TW", name: "繁體中文", og: "zh_TW" },
  { code: "ja", name: "日本語", og: "ja_JP" },
  { code: "es", name: "Español", og: "es_ES" },
  { code: "pt-BR", name: "Português", og: "pt_BR" },
  { code: "de", name: "Deutsch", og: "de_DE" },
  { code: "fr", name: "Français", og: "fr_FR" },
  { code: "ru", name: "Русский", og: "ru_RU" },
  { code: "hi", name: "हिन्दी", og: "hi_IN" },
  { code: "ar", name: "العربية", og: "ar_AR", rtl: true },
  { code: "tr", name: "Türkçe", og: "tr_TR" },
  { code: "id", name: "Bahasa Indonesia", og: "id_ID" },
];
const urlFor = (c) => (c === "en" ? BASE : BASE + c + "/");
const stripTags = (s) => (s || "").replace(/<[^>]+>/g, "").replace(/\s+/g, " ").trim();

const template = fs.readFileSync(path.join(here, "site-template.html"), "utf8");
const dicts = {};
for (const l of LOCALES) {
  dicts[l.code] = JSON.parse(fs.readFileSync(path.join(DOCS, "i18n", l.code + ".json"), "utf8"));
}

function ldBlock(obj) {
  return '<script type="application/ld+json">\n      ' +
    JSON.stringify(obj, null, 2).replace(/\n/g, "\n      ") + "\n    </script>";
}

let built = 0;
for (const l of LOCALES) {
  const d = dicts[l.code];
  const isRoot = l.code === "en";
  const ap = isRoot ? "" : "../";
  const $ = load(template);

  // Localized text / inline-HTML
  $("[data-i18n]").each((_, el) => { const k = $(el).attr("data-i18n"); if (d[k] != null) $(el).text(d[k]); });
  $("[data-i18n-html]").each((_, el) => { const k = $(el).attr("data-i18n-html"); if (d[k] != null) $(el).html(d[k]); });

  // Document language / direction
  $("html").attr("lang", l.code).attr("dir", l.rtl ? "rtl" : "ltr");

  // Title + meta + OG/Twitter (localized)
  $("title").text(d["meta.title"]);
  $('meta[name="description"]').attr("content", d["meta.desc"]);
  $('meta[property="og:title"]').attr("content", d["meta.title"]);
  $('meta[property="og:description"]').attr("content", d["meta.desc"]);
  $('meta[property="og:url"]').attr("content", urlFor(l.code));
  $('meta[name="twitter:title"]').attr("content", d["meta.title"]);
  $('meta[name="twitter:description"]').attr("content", d["meta.desc"]);
  if ($('meta[property="og:locale"]').length) $('meta[property="og:locale"]').attr("content", l.og);
  else $('meta[property="og:type"]').before('<meta property="og:locale" content="' + l.og + '" />\n    ');

  // Canonical (self) + hreflang alternates
  $('link[rel="canonical"]').attr("href", urlFor(l.code));
  $('link[rel="alternate"][hreflang]').remove();
  let alts = "";
  for (const a of LOCALES) alts += '\n    <link rel="alternate" hreflang="' + a.code + '" href="' + urlFor(a.code) + '" />';
  alts += '\n    <link rel="alternate" hreflang="x-default" href="' + BASE + '" />';
  $('link[rel="canonical"]').after(alts);

  // Depth-correct asset paths
  $('link[rel="icon"]').attr("href", ap + "favicon.svg");
  $(".hero-shot img").attr("src", ap + "hero.webp");
  $("script[src]").each((_, el) => { const s = $(el).attr("src") || ""; if (/i18n\.js|site\.js/.test(s)) $(el).attr("src", ap + "site.js"); });

  // Language switcher: current label + baked links
  $("#lang-current").text(l.name);
  const items = LOCALES.map((a) => {
    const href = a.code === "en" ? (isRoot ? "./" : "../") : (isRoot ? a.code + "/" : "../" + a.code + "/");
    const cur = a.code === l.code ? ' aria-current="true"' : "";
    return '<a class="lang-item" href="' + href + '" hreflang="' + a.code + '" dir="' + (a.rtl ? "rtl" : "ltr") + '"' + cur +
      "><span>" + a.name + '</span><span class="tick">✓</span></a>';
  }).join("");
  $("#lang-menu").html(items);

  // Serialize, then swap the two JSON-LD blocks (raw JSON, no HTML-escaping)
  const app = {
    "@context": "https://schema.org", "@type": "SoftwareApplication",
    name: "BigJSON", alternateName: "Huge JSON Viewer",
    operatingSystem: "macOS 11+, Windows 10+", applicationCategory: "DeveloperApplication",
    description: stripTags(d["meta.desc"]), url: urlFor(l.code),
    downloadUrl: "https://github.com/bandusix/huge-json-viewer/releases/latest",
    softwareVersion: "0.3.0", license: "https://opensource.org/licenses/MIT",
    author: { "@type": "Person", name: "bandusix" },
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    screenshot: BASE + "screenshot-dark.png",
    inLanguage: LOCALES.map((x) => x.code),
  };
  const faq = {
    "@context": "https://schema.org", "@type": "FAQPage",
    mainEntity: [1, 2, 3, 4, 5, 6, 7].map((i) => ({
      "@type": "Question", name: stripTags(d["faq.q" + i]),
      acceptedAnswer: { "@type": "Answer", text: stripTags(d["faq.a" + i]) },
    })),
  };
  const blocks = [ldBlock(app), ldBlock(faq)];
  let idx = 0;
  let html = $.html().replace(/<script type="application\/ld\+json">[\s\S]*?<\/script>/g, () => blocks[idx++] || "");

  const outDir = isRoot ? DOCS : path.join(DOCS, l.code);
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "index.html"), html);
  built++;
}

// sitemap.xml with per-URL hreflang alternates
let sm = '<?xml version="1.0" encoding="UTF-8"?>\n' +
  '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n';
for (const l of LOCALES) {
  sm += "  <url>\n    <loc>" + urlFor(l.code) + "</loc>\n";
  for (const a of LOCALES) sm += '    <xhtml:link rel="alternate" hreflang="' + a.code + '" href="' + urlFor(a.code) + '"/>\n';
  sm += '    <xhtml:link rel="alternate" hreflang="x-default" href="' + BASE + '"/>\n';
  sm += "    <lastmod>" + LASTMOD + "</lastmod>\n    <changefreq>weekly</changefreq>\n";
  sm += "    <priority>" + (l.code === "en" ? "1.0" : "0.8") + "</priority>\n  </url>\n";
}
sm += "</urlset>\n";
fs.writeFileSync(path.join(DOCS, "sitemap.xml"), sm);

console.log("Built " + built + " localized pages + sitemap.xml (" + LOCALES.length + " locales)");
