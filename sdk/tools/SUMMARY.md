# sdk/tools

- `package.json` — TypeDoc, TypeScript 6 (its compiler API), publint, are-the-types-wrong; pinned
- `bun.lock` — this folder's own lockfile (not part of the workspace)
- `typedoc.json` — API reference settings: the three entry points, output `site/api/`
- `api-index.md` — the landing text of the API reference
- `api-theme.css` — maps TypeDoc's theme onto the site's colors, light and dark
- `docs.sh` — `bun run docs` (generate) and `bun run docs:check` (fail when `site/api/` is stale)
