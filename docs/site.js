/* BigJSON site — language dropdown + first-visit auto-detect.
   Content is pre-rendered per language (SEO); this only toggles the menu and,
   from the English root only, redirects a fresh visitor to their language page. */
(function () {
  "use strict";
  var btn = document.getElementById("lang-btn");
  var menu = document.getElementById("lang-menu");

  function toggle(show) {
    if (!menu) return;
    var s = show == null ? menu.hidden : show;
    menu.hidden = !s;
    if (btn) btn.setAttribute("aria-expanded", String(s));
  }
  if (btn && menu) {
    btn.addEventListener("click", function (e) { e.stopPropagation(); toggle(); });
    document.addEventListener("click", function (e) {
      if (!menu.hidden && !menu.contains(e.target) && !btn.contains(e.target)) toggle(false);
    });
    document.addEventListener("keydown", function (e) { if (e.key === "Escape") toggle(false); });
    menu.addEventListener("click", function (e) {
      var a = e.target.closest("a[hreflang]");
      if (a) { try { localStorage.setItem("lang", a.getAttribute("hreflang")); } catch (_) {} }
    });
  }

  // Auto-detect: only from the English/root page, only on a fresh visit
  // (no saved choice, no query string). Googlebot (en-US) never redirects.
  if (document.documentElement.lang === "en") {
    var saved = null;
    try { saved = localStorage.getItem("lang"); } catch (_) {}
    if (!saved && !location.search) {
      var SUP = { "zh-CN": 1, "zh-TW": 1, "ja": 1, "es": 1, "pt-BR": 1, "de": 1, "fr": 1, "ru": 1, "hi": 1, "ar": 1, "tr": 1, "id": 1 };
      var langs = navigator.languages || [navigator.language || "en"];
      var target = null;
      for (var i = 0; i < langs.length && !target; i++) {
        var raw = langs[i], low = raw.toLowerCase();
        if (SUP[raw]) target = raw;
        else if (low === "zh-tw" || low === "zh-hk" || low === "zh-mo" || low.indexOf("zh-hant") === 0) target = "zh-TW";
        else if (low.indexOf("zh") === 0) target = "zh-CN";
        else if (low.indexOf("pt") === 0) target = "pt-BR";
        else {
          var base = low.split("-")[0];
          for (var k in SUP) { if (k.split("-")[0] === base) { target = k; break; } }
        }
      }
      if (target) location.replace(target + "/");
    }
  }
})();
