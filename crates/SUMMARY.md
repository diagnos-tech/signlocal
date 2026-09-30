# crates

Each crate has one responsibility, a README.md and, when pure or contract-tested, a SPEC.md.

- `websign-core/` — pure signing logic and display rules
- `websign-devices/` — USB and PC/SC devices, live events, devices.json hints
- `websign-host/` — the session engine
- `websign-i18n/` — localization engine
- `websign-keystores/` — key sources behind one trait
- `websign-project/` — project.toml identifiers as constants (Apache-2.0)
- `websign-protocol/` — the wire contract (Apache-2.0)
- `websign-registration/` — browser registration and status
- `websign-ui-model/` — pure view logic of both windows
