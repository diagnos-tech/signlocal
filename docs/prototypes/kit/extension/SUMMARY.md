# extension

Minimal Manifest V3 extension used by the native messaging proofs; it is a test harness, not the product extension.

- `background.js` — service worker: the only part that talks to the host; validates every request and takes the site from the browser's `sender`, never from the payload
- `config.js` — generated from `project.toml` by `generate-manifest.mjs`; do not edit
- `content.js` — announces the extension to the page and relays same-window messages of a known shape to the background
- `generate-manifest.mjs` — writes `manifest.json` and `config.js` from `project.toml`
- `manifest.json` — generated extension manifest (native messaging permission, localhost content script, fixed key for a stable extension ID)
- `native-host.js` — one native messaging port, opened on demand and closed after a minute of inactivity
- `project-config.mjs` — reads the identifiers the extension and its tests need from `project.toml`
