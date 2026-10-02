# app/src/logging/redact

- `network.rs` — IPv4 and IPv6 addresses (a `:port` stays).
- `paths.rs` — The home folder becomes `~`, the login name and other accounts' profile folders `[user]`, share servers `[host]`.
- `runs.rs` — Long digit, hex and Base64 runs: document numbers, digests, serials, encoded certificates.
- `spans.rs` — Replacing byte ranges of a message, shared by every pass.
- `structured.rs` — PEM blocks, sites, e-mails, distinguished-name values and `key=value` secrets or names.
