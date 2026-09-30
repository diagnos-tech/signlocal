# crates/websign-host/src/runtime

- `key_worker.rs` — The thread that owns every key store.
- `mod.rs` — Real threads around the engine: the stdio reader, the key store worker, the device monitor and the tick timer.
- `stdio.rs` — Frames over the process's stdin/stdout.
