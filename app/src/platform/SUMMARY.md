# app/src/platform

- `linux/` — Linux implementations of the platform functions
- `macos/` — macOS implementations of the platform functions
- `windows/` — Windows implementations of the platform functions
- `appearance.rs` — Light or dark as the OS is set, for where winit does not know (`docs/ux.md` §4.1).
- `caller.rs` — Identifying the program that started `websign connect`/`sign`/`choose`.
- `channel.rs` — Direct or store build and the install format, decided at run time (`docs/plan.md` D10).
- `focus.rs` — Bringing the confirmation window to the front without stealing focus (`docs/ux.md` §4.1).
- `mod.rs` — OS facts and actions the app needs outside key stores and registration.
- `motion.rs` — "Reduce motion" (`docs/ux.md` §11.4): `SPI_GETCLIENTAREAANIMATION`, `accessibilityDisplayShouldReduceMotion`, GNOME `enable-animations`.
- `url_events.rs` — `websign:` URLs delivered as OS events (macOS Apple Events) instead of arguments.
- `system_ui.rs` — OS windows the app hands work to: the certificate viewer and the .pfx import (`docs/ux.md` §5.12, §8.5), and opening https URLs.
