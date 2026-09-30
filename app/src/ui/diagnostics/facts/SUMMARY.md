# app/src/ui/diagnostics/facts

- `mod.rs` — What a scan found, as plain data the tabs, lights and report read.
- `collect.rs` — One full scan (key stores, USB, PC/SC, browsers, records) on a worker thread; `ScanInput::current`.
- `browsers.rs` — Installed browsers, their registration and the extension's last connection; report names.
- `devices.rs` — Tokens, cards and readers, and whether each brought certificates (links, possible devices, hints).
- `drivers.rs` — PKCS#11 modules: loaded or failed, found automatically or added by the person.
- `certificates.rs` — The Certificates tab's list from every key source, with the confirmation list's rules.
- `hidden_rows.rs` — Rows for hidden certificates still shown under "Can't sign" (login-only, no private key).
- `home.rs` — Paths without the user's folder (`~`, `%USERPROFILE%`).
- `system.rs` — Packaging name and OS description for the report.
