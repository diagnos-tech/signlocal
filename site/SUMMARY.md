# site

Static website for GitHub Pages (no build step). Pages are plain HTML with English text; `assets/i18n.js` swaps in the visitor's language from `locales/`.

- `index.html` — what WebeSign is, how it works, privacy promises
- `download.html` — install commands per OS, unsigned-build notices, checksum check
- `privacy.html` — what stays local, what a site receives, no trackers
- `developers.html` — SDK, Node and Rust client quickstarts
- `assets/` — tokens, styles, language switcher, icon
- `locales/` — UI text in en, es, pt-PT, pt-BR, fr, it, de
- `.nojekyll` — tells GitHub Pages to serve files as they are
