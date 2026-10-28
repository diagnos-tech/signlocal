# site/src/platform

What the wizard knows about the world, as pure functions shared by the build and the browser.

- `artifacts.ts` — the file names of a release and their drift guard against release.yml
- `catalog.ts` — browsers per system, processor and distribution questions
- `commands.ts` — the install, verify and uninstall commands (not translated)
- `detect.ts` — guesses system, processor, browser and distribution from the user agent
- `facts.ts` — compatibility facts no product file encodes
- `navigator.ts` — reads `navigator` for detect.ts and refines it with client hints
- `stores.ts` — store or developer install, per browser, from project.toml's IDs
- `types.ts` — the wizard's vocabulary
