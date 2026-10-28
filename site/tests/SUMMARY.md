# site/tests

Tests of the site. Unit tests run with `bun test`; end-to-end tests with Playwright against the built site.

- `e2e/` — Playwright: pages, accessibility (axe), wizard, languages, no-script, live status, phones, no release, screenshots
- `fixtures/` — release data, user agents and signature vectors
- `unit/` — unit tests of the pure logic, the data generators and the texts
