// Picks the language (?lang, then the saved choice, then the browser), loads
// locales/<lang>.json and fills every [data-i18n] element. English text is in
// the HTML, so the page reads correctly without JavaScript.
(function () {
  var LANGS = ["en", "es", "pt-PT", "pt-BR", "fr", "it", "de"];
  var select = document.getElementById("lang");

  function saved() {
    try { return localStorage.getItem("lang"); } catch (e) { return null; }
  }
  function save(lang) {
    try { localStorage.setItem("lang", lang); } catch (e) { /* private mode */ }
  }
  function match(tag) {
    if (!tag) return null;
    var t = tag.toLowerCase();
    for (var i = 0; i < LANGS.length; i++) if (LANGS[i].toLowerCase() === t) return LANGS[i];
    var base = t.split("-")[0];
    if (base === "pt") return t === "pt-pt" ? "pt-PT" : "pt-BR";
    for (var j = 0; j < LANGS.length; j++) if (LANGS[j] === base) return LANGS[j];
    return null;
  }
  function initial() {
    var q = new URLSearchParams(location.search).get("lang");
    var found = match(q) || match(saved());
    if (found) return found;
    var prefs = navigator.languages || [navigator.language];
    for (var i = 0; i < prefs.length; i++) if ((found = match(prefs[i]))) return found;
    return "en";
  }
  function apply(lang) {
    fetch("locales/" + lang + ".json")
      .then(function (r) { return r.ok ? r.json() : Promise.reject(r.status); })
      .then(function (dict) {
        document.documentElement.lang = lang;
        document.querySelectorAll("[data-i18n]").forEach(function (el) {
          var text = dict[el.getAttribute("data-i18n")];
          if (text) el.textContent = text;
        });
        if (select) select.value = lang;
      })
      .catch(function () { /* keep the English text */ });
  }

  var lang = initial();
  if (select) {
    select.value = lang;
    select.addEventListener("change", function () { save(select.value); apply(select.value); });
  }
  if (lang !== "en") apply(lang);
})();
