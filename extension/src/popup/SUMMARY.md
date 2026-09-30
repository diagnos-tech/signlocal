# extension/src/popup

- `card.ts` — What each popup state says and offers (docs/ux.md section 9 table): text, links and actions.
- `icons.ts` — The popup's inline SVG icons.
- `state.ts` — The popup's states (docs/ux.md section 9), decided purely from the probe result.
- `view.ts` — Renders the popup into `root` (plain DOM, no framework; whole popup < 15 KB).
