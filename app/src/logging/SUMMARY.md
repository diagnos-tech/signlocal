# app/src/logging

- `redact/` — the privacy filter's passes
- `location.rs` — Where the log lives: the OS's per-user log folder, never the shared temp folder.
- `redact.rs` — The last line of defense for log privacy: removes certificates, numbers, digests, e-mails, sites, IP addresses, labelled secrets and names, home folder, login, other profiles and share servers.
- `sink.rs` — The log file, size-capped by rotation (two files of at most 1 MiB, rotation shared by concurrent processes), private to the user.
- `tests.rs` — The log privacy audit: personal-looking data through the real `log` macros must not come out.
