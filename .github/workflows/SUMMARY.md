# .github/workflows

- `ci.yml` — required gates on every push and pull request: Rust on three OSes, keystore contract, xtask checks, TypeScript, installers
- `e2e.yml` — end-to-end matrix and screenshots; skipped until the e2e hooks are implemented
- `prototypes.yml` — Phase-0 proofs: builds the kit and exercises every signing path with software keys
- `release.yml` — `v*` tag → prerelease with every artifact, SHA256SUMS, install scripts and the release page
- `scheduled.yml` — weekly mutation testing and nightly fuzzing
- `ui-spike.yml` — Phase-0 proof 5: real-window and headless screenshots on seven systems
