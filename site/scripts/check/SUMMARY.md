# site/scripts/check

Gates that run on the built site (`bun run verify`).

- `budget.ts` — gzip budgets per page: HTML, JavaScript, CSS, font files
- `csp.ts` — the Content-Security-Policy rules: no inline script or style, no other origin; no folder notes in dist/
- `html.ts` — reads the facts the checks need out of a built page
- `links.ts` — every link and fragment resolves; the paths other code links to exist; one anchor per error code
- `run.ts` — runs the four checks (and, with --deploy, refuses sample release data)
- `seo.ts` — one h1, unique titles and descriptions, canonical and language alternates, noindex only on /activate/
