# crates/websign-host/src/runtime

- `softhsm_tests/` — the runtime serving a whole signature against a SoftHSM2 token
- `device_label.rs` — The device a key lives on, named without personal data.
- `devices.rs` — The reader and card monitor, feeding the engine.
- `key_worker.rs` — The thread that owns every key store.
- `mod.rs` — Real threads around the engine: the stdio reader, the key store worker, the device monitor and the tick timer.
- `possible.rs` — "Possible certificates" for the window: plugged-in tokens and cards that brought no certificate.
- `slow_listing.rs` — "Still reading {device}": a listing past 2 s is announced with a safe device name.
- `snapshot.rs` — From what the key sources found to what the engine and the window list.
- `stdio.rs` — Frames over the process's stdin/stdout.
