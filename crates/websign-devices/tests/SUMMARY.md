# crates/websign-devices/tests

- `devices_schema.rs` — validates `devices.json` against `devices.schema.json` (built-in validator for the keywords the schema uses).
- `monitor.rs` — the monitor against the local PC/SC service (`pcscd --disable-polkit` in containers); start-up, service state, prompt shutdown.
