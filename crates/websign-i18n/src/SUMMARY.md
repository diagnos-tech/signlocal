# crates/websign-i18n/src

- `catalog.rs` — Loaded messages for one locale.
- `check.rs` — Consistency rules between a locale file and the reference, used by `cargo xtask check i18n` and by this crate's tests.
- `dates.rs` — Dates and times per locale, without ICU.
- `lib.rs` — Translated text for the app, generated key constants, plural rules and date formatting (`docs/architecture/i18n.md`).
- `locale.rs` — The supported locales and how one is picked.
- `message.rs` — A message template being filled with arguments.
- `plural.rs` — CLDR plural rules for the shipped locales, as one pure function.
