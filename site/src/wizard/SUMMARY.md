# site/src/wizard

The install wizard. `state`, `url`, `conditions` and `recipe` are pure and unit-tested; the rest touches the page.

- `conditions.ts` — data-os, data-browser … conditions: which variants show
- `data.ts` — the JSON block of devices, channels and minimum app version
- `dom.ts` — typed element lookups
- `focus.ts` — focus management and screen-reader announcements
- `history.ts` — what the address bar says about a state (only confirmed choices)
- `live-status.ts` — chips and the ready banner from the SDK's status()
- `main.ts` — wires state, history, view, detection and sharing together
- `recipe.ts` — which cards the result shows, in order; the default install tab
- `result-view.ts` — the result's title and the other-computer notice
- `share.ts` — the link to the current steps
- `state.ts` — the reducer: steps, choices, preselection, notices, errors
- `strings.ts` — the message keys the wizard reads in the browser
- `test-hooks.ts` — installs the SDK fake for end-to-end builds only
- `url.ts` — the URL contract: parse and serialize
- `view.ts` — draws a state onto the page
