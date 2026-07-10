/* BigJSON site i18n — browser auto-detect + switcher + RTL.
   English is the inline default (SEO / no-JS); other locales load from i18n/<code>.json. */
(function () {
  "use strict";

  var LOCALES = [
    { code: "en", name: "English" },
    { code: "zh-CN", name: "简体中文" },
    { code: "ja", name: "日本語" },
    { code: "es", name: "Español" },
    { code: "pt-BR", name: "Português" },
    { code: "de", name: "Deutsch" },
    { code: "fr", name: "Français" },
    { code: "ru", name: "Русский" },
    { code: "hi", name: "हिन्दी" },
    { code: "ar", name: "العربية", rtl: true },
    { code: "tr", name: "Türkçe" },
    { code: "id", name: "Bahasa Indonesia" },
  ];
  var RTL = { ar: true };
  var byCode = {};
  LOCALES.forEach(function (l) { byCode[l.code] = l; });

  var enCache = null;

  function detect() {
    var saved = localStorage.getItem("lang");
    if (saved && byCode[saved]) return saved;
    var langs = navigator.languages || [navigator.language || "en"];
    for (var i = 0; i < langs.length; i++) {
      var raw = langs[i];
      var low = raw.toLowerCase();
      if (byCode[raw]) return raw;
      if (low.indexOf("zh") === 0) return "zh-CN";
      if (low.indexOf("pt") === 0) return "pt-BR";
      var base = low.split("-")[0];
      for (var j = 0; j < LOCALES.length; j++) {
        if (LOCALES[j].code.split("-")[0].toLowerCase() === base) return LOCALES[j].code;
      }
    }
    return "en";
  }

  function snapshotEnglish() {
    if (enCache) return;
    enCache = { __title: document.title };
    var dm = document.querySelector('meta[name="description"]');
    enCache.__desc = dm ? dm.content : "";
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      enCache["t:" + el.getAttribute("data-i18n")] = el.textContent;
    });
    document.querySelectorAll("[data-i18n-html]").forEach(function (el) {
      enCache["h:" + el.getAttribute("data-i18n-html")] = el.innerHTML;
    });
  }

  function apply(dict, code) {
    var rtl = !!RTL[code];
    document.documentElement.lang = code;
    document.documentElement.dir = rtl ? "rtl" : "ltr";
    var en = enCache || {};
    document.querySelectorAll("[data-i18n]").forEach(function (el) {
      var key = el.getAttribute("data-i18n");
      var v = dict && dict[key] != null ? dict[key] : en["t:" + key];
      if (v != null) el.textContent = v;
    });
    document.querySelectorAll("[data-i18n-html]").forEach(function (el) {
      var key = el.getAttribute("data-i18n-html");
      var v = dict && dict[key] != null ? dict[key] : en["h:" + key];
      if (v != null) el.innerHTML = v;
    });
    var title = (dict && dict["meta.title"]) || en.__title;
    if (title) document.title = title;
    var dm = document.querySelector('meta[name="description"]');
    var desc = (dict && dict["meta.desc"]) || en.__desc;
    if (dm && desc) dm.content = desc;
    var cur = document.getElementById("lang-current");
    if (cur) cur.textContent = (byCode[code] || byCode.en).name;
    document.querySelectorAll(".lang-item").forEach(function (it) {
      it.setAttribute("aria-current", String(it.getAttribute("data-code") === code));
    });
  }

  function setLang(code) {
    if (!byCode[code]) code = "en";
    try { localStorage.setItem("lang", code); } catch (e) {}
    if (code === "en") { apply(null, "en"); return; }
    fetch("i18n/" + code + ".json", { cache: "default" })
      .then(function (r) { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then(function (dict) { apply(dict, code); })
      .catch(function (e) { console.warn("i18n load failed:", code, e); apply(null, "en"); });
  }

  function toggle(show) {
    var menu = document.getElementById("lang-menu");
    var btn = document.getElementById("lang-btn");
    var willShow = show == null ? menu.hidden : show;
    menu.hidden = !willShow;
    btn.setAttribute("aria-expanded", String(willShow));
  }

  function init() {
    snapshotEnglish();
    var menu = document.getElementById("lang-menu");
    var btn = document.getElementById("lang-btn");
    menu.innerHTML = LOCALES.map(function (l) {
      return '<button class="lang-item" data-code="' + l.code + '" dir="' + (l.rtl ? "rtl" : "ltr") + '">' +
        "<span>" + l.name + '</span><span class="tick">✓</span></button>';
    }).join("");
    menu.addEventListener("click", function (e) {
      var it = e.target.closest(".lang-item");
      if (it) { setLang(it.getAttribute("data-code")); toggle(false); }
    });
    btn.addEventListener("click", function (e) { e.stopPropagation(); toggle(); });
    document.addEventListener("click", function (e) {
      if (!menu.hidden && !menu.contains(e.target) && !btn.contains(e.target)) toggle(false);
    });
    document.addEventListener("keydown", function (e) { if (e.key === "Escape") toggle(false); });
    setLang(detect());
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
