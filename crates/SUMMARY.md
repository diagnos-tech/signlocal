# crates

Each crate has one responsibility, a README.md and, when pure or contract-tested, a SPEC.md.

- `signlocal-core/` — pure signing logic and display rules
- `signlocal-devices/` — USB and PC/SC devices, live events, devices.json hints
- `signlocal-host/` — the session engine
- `signlocal-i18n/` — localization engine
- `signlocal-keystores/` — key sources behind one trait
- `signlocal-project/` — project.toml identifiers as constants (Apache-2.0)
- `signlocal-protocol/` — the wire contract (Apache-2.0)
- `signlocal-registration/` — browser registration and status
- `signlocal-ui-model/` — pure view logic of both windows
