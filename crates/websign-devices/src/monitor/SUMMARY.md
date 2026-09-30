# crates/websign-devices/src/monitor

- `session.rs` — the monitor thread: PC/SC context lifecycle, status-change loop, service outages.
- `tracker.rs` — pure diff of reader views into reader/card events.
