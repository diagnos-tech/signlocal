# site

The signlocal website (signlocal.diagnos.health): Astro pages in seven languages, built from the repository's own sources and published to GitHub Pages by `.github/workflows/pages.yml`.

- `astro.config.ts` — Astro settings: static output, file names kept as written, aliases to the SDK sources, the sitemap
- `bun.lock` — resolved dependency versions (the site is its own package, not a workspace member)
- `lighthouserc.json` — Lighthouse CI: the pages measured and the scores required
- `package.json` — scripts and pinned dev dependencies
- `playwright.config.ts` — end-to-end settings: Chromium against the built site served like GitHub Pages
- `public/` — files copied as they are: icons, social image, licence texts, the built boot script
- `README.md` — what the site is, how to run, check and deploy it, its data sources and the paths other code links to
- `scripts/` — build-time generators, post-build checks, the preview server and the image renderer
- `src/` — pages, components, client code, texts and the pure logic of the install wizard
- `tests/` — unit tests, end-to-end tests and their fixtures
- `tsconfig.json` — TypeScript settings (Astro strict plus the repository's strict flags)
