# extension/src/popup

- `card.ts` — What each popup state says and offers (docs/ux.md section 9 table): text, links and actions.
- `dom.ts` — DOM building blocks: elements, buttons, links that open tabs, and the status block.
- `fixture.ts` — Screenshot fixtures: answers from the query string; compiled only by `wxt build --mode fixtures`.
- `i18n.ts` — Text lookup that fills placeholders by name (the catalogs number them alphabetically).
- `icons.ts` — The popup's inline SVG icons.
- `links.ts` — Site pages the popup opens: download per OS, finish setup, test, privacy.
- `state.ts` — The popup's states (docs/ux.md section 9), decided purely from the probe result.
- `view.ts` — Renders the popup into `root`: a fixed frame whose status, actions and footer change (whole popup < 15 KB).
