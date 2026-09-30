# probe/src

Source of `websign-probe`, organized by concept.

- `cli.rs` — command-line entry point; each subcommand owns its arguments
- `commands/` — the subcommands that work across key sources
- `config.rs` — identifiers from `project.toml`, injected by `build.rs`
- `devices/` — USB devices and smart card readers that might hold a certificate
- `keystores/` — key sources: Windows, macOS, and PKCS#11
- `main.rs` — program entry: dispatches to the CLI or to the native messaging host
- `nm/` — native messaging: host loop, protocol, and browser registration
- `platform/` — small OS facts kept apart from key handling
- `trace.rs` — opt-in progress trace on stderr (`WEBSIGN_PROBE_TRACE=1`)
