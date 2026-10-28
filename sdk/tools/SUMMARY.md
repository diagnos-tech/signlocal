# sdk/tools

- `package.json` — TypeDoc, TypeScript 6 (its compiler API), publint, are-the-types-wrong; pinned
- `bun.lock` — this folder's own lockfile (not part of the workspace)
- `typedoc.json` — API reference settings: the three entry points, output `site/dist/api/`
- `api-index.md` — the landing text of the API reference
- `api-theme.css` — maps TypeDoc's theme onto the site's light palette (the reference is light only, like the site)
- `docs.sh` — `bun run docs` writes the API reference to `site/dist/api/`; `--out <dir>` writes it elsewhere
