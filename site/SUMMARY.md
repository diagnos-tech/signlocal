# site

Static website for GitHub Pages, deployed by `.github/workflows/pages.yml`. Pages are plain HTML with English text; `assets/i18n.js` swaps in the visitor's language from `locales/`.

- `index.html` — what WebeSign is, how it works, privacy promises
- `download.html` — install commands per OS, unsigned-build notices, checksum check
- `privacy.html` — what stays local, what a site receives, no trackers
- `developers.html` — SDK, Node and Rust client quickstarts
- `activate/` — "Finish setting up WebeSign": button to `websign:activate`, fallback pointing to the download page
- `test/` — test-signature page: detects extension and app, signs a sample text, verifies it in the browser (nothing is sent anywhere)
- `README.md` — purpose, preview, deploy and the maintainer steps left
- `build-sdk.sh` — rebuilds `assets/websign-sdk.js` from `sdk/` (`bun run build` + `bun build --minify`); committed, and rebuilt again at deploy
- `tests/` — Node/bun unit tests of the key extractor and verifier with openssl vectors (`bun test site/tests`)
- `assets/` — tokens, styles, language switcher, icon, SDK bundle, test page scripts
- `locales/` — UI text in en, es, pt-PT, pt-BR, fr, it, de
- `.nojekyll` — tells GitHub Pages to serve files as they are
