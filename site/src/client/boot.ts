// The first script of every page (public/boot.js, blocking, under a kilobyte):
// marks the page as scripted and sends visitors to their language.
//
//  - ?lang=<code> opens that language's copy of the page and remembers the choice.
//  - Otherwise, on the English copy of a translated page only, the saved choice or the browser's
//    languages decide; pages under a language folder never redirect, so a shared Portuguese
//    link stays Portuguese. English-only pages never redirect at all.
// Query and hash travel with the redirect (the popup's download.html#macos keeps working).
// Everything is guarded: a failure here must never break the page.

import { choose, matchTag } from "../i18n/choose.ts";
import { folderOf } from "../i18n/codes.ts";

function run(): void {
  const root = document.documentElement;
  root.classList.add("js");
  const url = new URL(location.href);
  const asked = url.searchParams.get("lang");
  const chosen = matchTag(asked);
  const page = matchTag(root.lang) ?? "en";
  let saved: string | null = null;
  try {
    saved = localStorage.getItem("lang");
  } catch {}
  const wanted =
    chosen ?? (page === "en" ? choose({ saved, languages: navigator.languages }) : page);
  url.searchParams.delete("lang");
  const rest = url.search + url.hash;
  if (root.dataset.localized === "true" && wanted !== page) {
    const own = folderOf(page);
    const path = url.pathname.startsWith(`${own}/`) ? url.pathname.slice(own.length) : url.pathname;
    location.replace(folderOf(wanted) + path + rest);
  } else if (asked !== null) {
    history.replaceState(null, "", url.pathname + rest);
  }
  try {
    if (chosen) localStorage.setItem("lang", chosen);
  } catch {}
}

try {
  run();
} catch {}
