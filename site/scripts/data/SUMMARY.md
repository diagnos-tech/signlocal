# site/scripts/data

Generators run by `bun run data`; each fails, naming its source, when the source is missing or changed shape.

- `boot.ts` — builds public/boot.js from src/client/boot.ts and enforces its size
- `message-keys.ts` — writes the union type of every message key from en.toml
- `project.ts` — project.toml and the SDK's package files → project.json
- `release.ts` — the newest release from the GitHub Releases API with its SHA256SUMS → release.json
- `run.ts` — runs every generator; picks the release source from SIGNLOCAL_SITE_RELEASE
- `snippets.ts` — cuts the developers page's code samples out of examples/
- `support.ts` — minimum browser and macOS versions and the tested Linux distributions → support.json
