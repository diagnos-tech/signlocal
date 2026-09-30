// Turns the generic "Download" button into "Download for <your OS>".
//
// Every variant is already in the HTML (one span per OS, each with its own
// data-i18n key, because word order differs between languages); this only
// unhides the one that matches detect.js and points the link at that tab of
// the download page. Phones and unknown systems keep the generic label and see
// the "computers only" hint.
(function () {
  var os = (window.websignDetect || {}).os;
  var desktop = os === "windows" || os === "macos" || os === "linux";
  document.querySelectorAll("[data-when-os]").forEach(function (el) {
    var when = el.getAttribute("data-when-os");
    el.hidden = desktop ? when !== os : when !== "other" && when !== (os || "other");
  });
  document.querySelectorAll("[data-os-link]").forEach(function (a) {
    if (desktop) a.href = a.getAttribute("href").split("#")[0] + "#" + os;
  });
})();
