# crates/websign-host/src/store

- `documents/` — unit tests of the document rules
- `file/` — unit tests of `JsonFile`
- `connections.rs` — Every extension that connected (`hello` with `browser`), for the diagnostics Browsers tab ("Extension 1.4.2 connected · 3 min ago").
- `consent.rs` — Remembered callers (`docs/ux.md` §4.10): "Remember this site/program".
- `disk.rs` — The stores as JSON files in the per-user data folder (`SPEC.md` §7).
- `documents.rs` — The five documents and the rules that change them, independent of where they are kept.
- `errors.rs` — The last errors, for "Copy diagnostics" (`docs/ux.md` §8.7).
- `file.rs` — A versioned JSON document with locked, atomic updates.
- `memory.rs` — The stores in memory: for tests, and for a host that has no data folder.
- `mod.rs` — What the app remembers between runs, as small JSON files in the per-user data folder.
- `paths.rs` — Where the files live.
- `private_file.rs` — Files only their owner can read (`0600` in a `0700` folder on Unix).
- `settings.rs` — Preferences the person set in diagnostics.
- `usage.rs` — When each certificate was last used by anyone, for list order (`docs/ux.md` §5.9 rule 2).
