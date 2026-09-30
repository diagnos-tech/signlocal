# extension/scripts

- `playwright-types.ts` — The slice of Playwright's API that the screenshot script uses (Playwright lives in e2e/).
- `screenshots.ts` — Screenshots of every popup state (light, dark, each locale) into docs/screenshots/popup/.
- `size.ts` — Popup budget check for CI: popup.html plus the scripts and styles it loads stay under 15 KB.
