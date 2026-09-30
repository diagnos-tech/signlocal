# .github/workflows

- `ci.yml` — required gates on every push and pull request: Rust on three OSes, keystore contract, xtask checks, TypeScript, installers
- `e2e.yml` — end-to-end matrix and screenshots; skipped until the e2e hooks are implemented
- `prototypes.yml` — Phase-0 proofs: builds the kit and exercises every signing path with software keys
- `scheduled.yml` — weekly mutation testing and nightly fuzzing
- `ui-spike.yml` — Phase-0 proof 5: real-window and headless screenshots on seven systems
