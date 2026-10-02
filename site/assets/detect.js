// Guesses the visitor's operating system and browser so the pages can open
// the right download tab and install steps first. It only reads what the
// browser already exposes to every page and sends nothing anywhere; a wrong
// or missing guess just means the visitor picks the tab by hand.
(function () {
  var ua = navigator.userAgent || "";
  var hints = navigator.userAgentData || null;

  function os() {
    var platform = (hints && hints.platform) || navigator.platform || "";
    var text = platform + " " + ua;
    // Phones and tablets first: iPadOS reports "MacIntel" but has touch points.
    if (/Android|iPhone|iPad|iPod/i.test(text)) return "mobile";
    if (/Mac/i.test(platform) && navigator.maxTouchPoints > 1) return "mobile";
    if (/Win/i.test(text)) return "windows";
    if (/Mac/i.test(text)) return "macos";
    if (/Linux|X11|CrOS|BSD/i.test(text)) return "linux";
    return null;
  }

  function browser() {
    var brands = hints && hints.brands ? hints.brands.map(function (b) { return b.brand; }).join(" ") : "";
    if (navigator.brave || /Brave/.test(brands)) return "brave";
    if (/Edg\//.test(ua) || /Edge/.test(brands)) return "edge";
    if (/OPR\//.test(ua) || /Opera/.test(brands)) return "opera";
    if (/Firefox\//.test(ua)) return "firefox";
    if (/Chrome\/|Chromium\//.test(ua)) return "chrome";
    if (/Safari\//.test(ua)) return "safari";
    return null;
  }

  window.websignDetect = { os: os(), browser: browser() };
})();
