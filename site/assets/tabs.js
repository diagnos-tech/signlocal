// Accessible tabs (WAI-ARIA tabs pattern with automatic activation).
//
// Markup: a [data-tabs] element holding a [role=tablist] of buttons with
// data-tab="<id>" and panels with id="<id>". Without JavaScript every panel
// stays visible, so the page still shows every step. The first tab to open is
// the one named in the URL hash (download.html#macos), else the detected OS or
// browser (data-detect="os" | "browser", from detect.js), else the first.
(function () {
  function setup(root) {
    var tabs = Array.prototype.slice.call(root.querySelectorAll("[role=tab]"));
    if (!tabs.length) return;
    var detected = (window.websignDetect || {})[root.getAttribute("data-detect")];
    var wanted = location.hash.slice(1);

    function panel(tab) { return document.getElementById(tab.getAttribute("data-tab")); }
    function select(tab, focus) {
      tabs.forEach(function (t) {
        var on = t === tab;
        t.setAttribute("aria-selected", on ? "true" : "false");
        t.tabIndex = on ? 0 : -1;
        panel(t).hidden = !on;
      });
      if (focus) tab.focus();
    }
    function find(id) {
      for (var i = 0; i < tabs.length; i++) if (tabs[i].getAttribute("data-tab") === id) return tabs[i];
      return null;
    }

    tabs.forEach(function (tab, i) {
      tab.setAttribute("aria-controls", tab.getAttribute("data-tab"));
      panel(tab).setAttribute("role", "tabpanel");
      panel(tab).setAttribute("aria-labelledby", tab.id);
      tab.addEventListener("click", function () { select(tab, false); });
      tab.addEventListener("keydown", function (e) {
        var next = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: tabs.length - 1 }[e.key];
        if (next === undefined) return;
        e.preventDefault();
        select(tabs[(next + tabs.length) % tabs.length], true);
      });
    });
    root.classList.add("tabs-on");
    select(find(wanted) || find(detected) || tabs[0], false);
  }

  document.querySelectorAll("[data-tabs]").forEach(setup);
})();
