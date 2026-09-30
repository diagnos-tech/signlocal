# websign-devices

Hardware that may hold a certificate: USB tokens and readers (VID:PID, never
serial numbers), smart card readers and their ATR, live PC/SC events that
invalidate the certificate cache, driver hints from
[`devices.json`](../../devices.json), and the "possible certificates" rule.

- `usb` and `pcsc` are promoted from the Phase-0 kit.
- Contract: [`SPEC.md`](SPEC.md).
- Linux needs `libpcsclite` at run time (package dependency).
- License: GPL-3.0-or-later.
