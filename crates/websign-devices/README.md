# websign-devices

Hardware that may hold a certificate: USB tokens and readers (VID:PID, never
serial numbers), smart card readers and their ATR, live PC/SC events that
invalidate the certificate cache, driver hints from
[`devices.json`](../../devices.json), and the "possible certificates" rule.

- `usb` and `pcsc` are promoted from the Phase-0 kit.
- `monitor` reports reader/card events (used to invalidate the certificate cache); `hints` and `possible` turn VID:PID and ATR into driver suggestions from `devices.json`.
- Tests: `cargo test -p websign-devices`. The monitor test uses the local PC/SC service when there is one (in a container: `pcscd --foreground --disable-polkit`); without it the monitor reports `ServiceChanged{running:false}` and the test still passes.
- Contract: [`SPEC.md`](SPEC.md).
- Linux needs `libpcsclite` at run time (package dependency).
- License: GPL-3.0-or-later.
