# .github/workflows

- `ci.yml` — required gates on every push and pull request: Rust on three OSes, keystore contract, xtask checks, TypeScript, installers
- `e2e.yml` — end-to-end matrix and screenshots; skipped until the e2e hooks are implemented
- `pages.yml` — push to `main` / by hand → rebuilds the SDK bundle and deploys `site/` to GitHub Pages
- `prototypes.yml` — by hand only: risk proofs that build the prototype kit and exercise every signing path with software keys
- `release.yml` — `v*` tag → prerelease with every artifact, SHA256SUMS, install scripts and the release page
- `scheduled.yml` — weekly mutation testing and nightly fuzzing
- `ui-spike.yml` — by hand only: real-window and headless screenshot spike on seven systems
