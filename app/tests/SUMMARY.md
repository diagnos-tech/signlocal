# app/tests

- `cli_process.rs` — The built binary as callers see it: stdout, exit codes, install/uninstall (twice: idempotent) in a throwaway home, foreign extension arguments refused, `connect` ending with stdin.
- `connect_softhsm.rs` — `signlocal connect` through `signlocal-client` signing with a SoftHSM2 token (feature `e2e`, Linux).
- `release_build.rs` — The default build contains no e2e hook; an e2e build does (proving the search works).
