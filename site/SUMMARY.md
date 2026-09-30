# site

Static website for GitHub Pages, deployed by `.github/workflows/pages.yml`. Pages are plain HTML with English text; `assets/i18n.js` swaps in the visitor's language from `locales/`.

- `index.html` — landing page: tagline, download button for the visitor's OS, how it works, browsers and systems, privacy promises, FAQ
- `download.html` — app install per OS and extension steps per browser (tabs picked by detection), unsigned-build notices, checksum check
- `privacy.html` — what stays local, what a site receives, no trackers
- `developers.html` — SDK, Node and Rust client quickstarts
- `activate/` — "Finish setting up WebeSign": button to `websign:activate`, fallback pointing to the download page
- `test/` — test-signature page: detects extension and app, signs a sample text, verifies it in the browser (nothing is sent anywhere)
- `README.md` — purpose, preview, deploy and the maintainer steps left
- `build-sdk.sh` — rebuilds `assets/websign-sdk.js` from `sdk/` (`bun run build` + `bun build --minify`); committed, and rebuilt again at deploy
- `tests/` — Node/bun tests: key extractor and verifier with openssl vectors, locale keys (`bun test site/tests`)
- `api/` — generated API reference of `@websign/sdk` (TypeDoc, committed; `bun run docs` in `sdk/`)
- `assets/` — tokens, styles, language switcher, icon, SDK bundle, test page scripts
- `locales/` — UI text in en, es, pt-PT, pt-BR, fr, it, de
- `.nojekyll` — tells GitHub Pages to serve files as they are
