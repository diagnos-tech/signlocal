# app/src/platform

- `linux/` — Linux implementations of the platform functions
- `macos/` — macOS implementations of the platform functions
- `windows/` — Windows implementations of the platform functions
- `caller.rs` — Identifying the program that started `websign connect`/`sign`/`choose`.
- `channel.rs` — Direct or store build, decided at run time (`docs/plan.md` D10).
- `focus.rs` — Bringing the confirmation window to the front (`docs/ux.md` §4.1).
- `mod.rs` — OS facts and actions the app needs outside key stores and registration.
- `motion.rs` — "Reduce motion" (`docs/ux.md` §11.4): `SPI_GETCLIENTAREAANIMATION`, `accessibilityDisplayShouldReduceMotion`, GNOME `enable-animations`.
- `system_ui.rs` — OS windows the app hands work to: the certificate viewer and the .pfx import (`docs/ux.md` §5.12, §8.5), and opening URLs.
