# site/tests/unit

Unit tests (`bun test tests/unit`).

- `activate.test.ts` — the activate page's states
- `artifacts.test.ts` — release file names and the drift guard against release.yml
- `choose.test.ts` — language choice
- `commands.test.ts` — exact install and verify commands
- `conditions.test.ts` — the conditions truth table
- `crypto.test.ts` — key extraction and signature checks against openssl
- `detect.test.ts` — detection over the user-agent fixtures
- `facts.test.ts` — facts still stated in docs/architecture/compatibility.md
- `messages.test.ts` — locale parity and use of every key
- `preload-modules.test.ts` — the module preload hints written into each page after the build
- `recipe.test.ts` — the result's cards
- `release-data.test.ts` — release selection and consistency checks
- `rich.test.ts` — rich text escaping and markers
- `state.test.ts` — every transition of the wizard
- `stores.test.ts` — store or developer install per browser
- `support.test.ts` — the extractors against the real repository files
- `test-page.test.ts` — the test page's states and summary
- `url.test.ts` — the URL contract
