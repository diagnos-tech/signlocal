# signlocal website

[signlocal.diagnos.health](https://signlocal.diagnos.health/): what signlocal is, a guided install for one computer,
a test page, help, compatibility and developer documentation, in seven languages. Astro, static output, plain
TypeScript in the browser, no runtime dependency, no cookie, no third-party request.

| Path | Page |
|---|---|
| `/`, `/pt-br/`, `/pt-pt/`, `/es/`, `/fr/`, `/it/`, `/de/` | Home |
| `/download.html` | The install wizard: system → browser → certificate → the steps for this computer |
| `/test/` | Test your setup: signs a sample and checks it in the browser |
| `/activate/` | Finish setting up: connects the app to the browsers |
| `/compatibility.html`, `/help.html`, `/privacy.html` | Compatibility, help, privacy |
| `/developers.html`, `/api/`, `/notices.html` | For developers, API reference (TypeDoc), licenses (English only) |

Pages other than the English-only ones exist once per language under `/<language>/` (`/pt-br/download.html`).
A tiny blocking script (`public/boot.js`) sends visitors of the English root pages to their language.

## Commands

Bun 1.3.11 and Node 22 or later. The site is its own package (its own `bun.lock`, TypeScript 5.9 for Astro), not a
workspace member of the repository root.

```sh
cd site
bun install --frozen-lockfile
bun run dev           # data, then astro dev
bun run build         # data, Astro, then the API reference (TypeDoc) into dist/api
bun run preview       # dist/ on http://localhost:4173, served the way GitHub Pages serves it
bun run test          # unit tests (bun test tests/unit)
bun run verify        # post-build checks (links, security policy, SEO, size), then Playwright + axe
bun run lighthouse    # Lighthouse CI on seven pages: 95+ in every category
bun run check         # lint, astro check, unit tests, build, verify: what CI runs
bun run images        # re-renders public/og.png and public/apple-touch-icon.png
```

Playwright needs a browser: `bunx playwright install chromium`, or point `SIGNLOCAL_CHROME` at a Chrome or Chromium
binary. The live-status specs need a build made with `PUBLIC_SIGNLOCAL_TEST_HOOKS=1` (it compiles in the SDK's fake,
used only when the address says `?sdk-fake=<scenario>` on localhost); the published site never contains it.

## Where the content comes from

Nothing a person sees is a hand-kept copy of something the product already says:

| What | Source of truth | Read and turned into data by |
|---|---|---|
| Version, date, file sizes and SHA-256 of the install files | The newest GitHub release and its `SHA256SUMS` | `scripts/data/release.ts` |
| Names, URLs, store IDs, URL scheme, minimum app version | `project.toml` | `scripts/data/project.ts` |
| Store or developer install, per browser | The store IDs of `project.toml` (empty today) | `src/platform/stores.ts` |
| Minimum browser and macOS versions, tested Linux distributions | `extension/build/manifest.ts`, `packaging/macos/Info.plist.in`, `.github/workflows/e2e.yml` | `scripts/data/support.ts` |
| Tokens, cards, drivers and their downloads | `devices.json` | `src/lib/devices.ts` |
| Error codes and what to do | `sdk/src/generated/ErrorCode.ts`, `sdk/src/hints.ts`, `clients/node/src/hints.ts` | `ErrorsTable.astro` |
| Developer code samples | `// #region site:<name>` blocks in `examples/` (type-checked there) | `scripts/data/snippets.ts` |
| API reference | `sdk/` through TypeDoc | `sdk/tools/docs.sh` |
| Design tokens | `design/tokens.css` and `design/tokens-web.css` | imported directly |

`SIGNLOCAL_SITE_RELEASE` picks the release: `github` (Pages: any failure stops the build and the previous site stays
online), `fixture` (CI checks, offline work: a "sample release data" ribbon appears on every page), `none` (no release
published yet: the wizard says "the first release is almost ready"), or unset (GitHub, else the fixture with a warning).
Nothing about a release is committed. The Pages workflow refuses a build that shows the ribbon.

## Texts

`src/i18n/messages/<language>.toml`, one per language, `en.toml` being the reference (format:
`docs/architecture/i18n.md`: dotted keys, `{placeholders}`, translator notes as comments, no plurals). A test checks
that every language has the same keys and placeholders and that every key is used. Developer pages and the license
notices are English and are not in the TOML files.

## Deploying

`.github/workflows/pages.yml` builds and checks the site and deploys it to GitHub Pages on every push to `develop`,
once a day (so the release data stays fresh), by hand, and right after the release workflow publishes a release. The
custom domain is set in the repository's Pages settings (source: GitHub Actions; DNS and HTTPS are live); the build
also writes `dist/CNAME` for copies of the site published elsewhere. After a deploy, the workflow looks for its own
commit in `<meta name="signlocal:build">` on the live page and warns if the CDN still serves an older build.

## Paths other code links to

These keep working; the end-to-end suite and `scripts/check/links.ts` check them.

| Address | Linked from |
|---|---|
| `download.html` | `sdk/src/install.ts`, the app's browser diagnostics |
| `download.html#windows`, `#macos`, `#linux` | the extension popup (understood by the wizard, then written as `?os=`) |
| `activate/` | the extension popup, the app's getting-started checklist |
| `test/` | the extension popup, the app's getting-started checklist |
| `privacy.html` | the extension popup, the app's help |
| `developers.html#error-<Code>` | `SignlocalError.docsUrl` and the desktop clients' errors |
| `api/` | `sdk/README.md` |

The install page's address is a documented contract for integrators (`/developers.html#links`): `os`, `arch`, `distro`,
`browser`, `cert`, `device` and `step`, e.g. `download.html?cert=token`.

## Brand, fonts, icons

The mark is `src/brand/mark.svg` (everything refers to it). Fonts, icons and browser logos are self-hosted with their
sources and licences in `src/fonts/`, `src/icons/` and `src/brands/`; `/notices.html` shows them. Never create a
`developers/` or `download/` folder under `src/pages/`: it would shadow the `.html` page of the same name.

The two fonts are trimmed copies of the Fontsource files (weights 400 to 700, the characters the seven languages use);
`src/fonts/subset.sh` rebuilds them. Together with the module hints that `scripts/preload-modules.ts` writes into every
page after the build, this keeps the first screen inside the Lighthouse budget (largest contentful paint 2 s on the
default mobile profile).

See [`SUMMARY.md`](SUMMARY.md) for the folders.
