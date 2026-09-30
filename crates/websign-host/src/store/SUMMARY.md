# crates/websign-host/src/store

- `connections.rs` — Every extension that connected (`hello` with `browser`), for the diagnostics Browsers tab ("Extension 1.4.2 connected · 3 min ago").
- `consent.rs` — Remembered callers (`docs/ux.md` §4.10): "Remember this site/program".
- `errors.rs` — The last errors, for "Copy diagnostics" (`docs/ux.md` §8.7).
- `file.rs` — A versioned JSON document with locked, atomic updates.
- `mod.rs` — What the app remembers between runs, as small JSON files in the per-user data folder.
- `paths.rs` — Where the files live.
- `settings.rs` — Preferences the person set in diagnostics.
- `usage.rs` — When each certificate was last used by anyone, for list order (`docs/ux.md` §5.9 rule 2).
