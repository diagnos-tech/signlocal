# extension/public

Files WXT copies into the extension as they are.

- `_locales/` — `chrome.i18n` catalogs generated from `i18n/` by `cargo xtask gen`; do not edit
- `icons/` — placeholder toolbar and store icons: SVG masters and the PNGs `scripts/icons.ts` renders from them (final brand icon pending, see TODO(gustavo) in `icon.svg`)
