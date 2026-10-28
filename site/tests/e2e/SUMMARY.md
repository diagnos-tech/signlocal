# site/tests/e2e

Playwright specs (Chromium).

- `a11y.spec.ts` — axe: zero violations on every page, wizard step and result, in English and Portuguese
- `client-hints.ts` — the client hints (navigator.userAgentData) a real browser with a given user agent offers
- `helpers.ts` — user agents, page lists and wizard URLs
- `i18n.spec.ts` — language redirects, ?lang, the language menu
- `live-status.spec.ts` — chips and the ready banner with the SDK fake
- `mobile.spec.ts` — phones and Chromebooks
- `no-release.spec.ts` — the install page with no published release
- `nojs.spec.ts` — the pages without JavaScript
- `pages.spec.ts` — every page: status, console, requests, overflow at 360 px, skip link
- `screenshots.spec.ts` — full-page pictures for the visual review
- `test.ts` — the specs' `test`: an emulated user agent comes with that browser's client hints
- `wizard.spec.ts` — the wizard: keyboard, history, detection, notices, recipe, sharing
