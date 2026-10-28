# design

Design sources shared by the app, the extension popup and the site.

- `tokens.css` — the design tokens of `docs/ux.md` §11 (colors light/dark, type, spacing, motion); the app's theme is tested against it
- `tokens-web.css` — web-only tokens (`--web-*`: ink ramp, surfaces, type scale, space, shadows) layered on `tokens.css` for the site; light only, no Rust counterpart
