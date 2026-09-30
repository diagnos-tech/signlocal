# websign-i18n

Localization for the app: compile-time message keys generated from
[`i18n/en.toml`](../../i18n/en.toml), the seven embedded locales, fallback
chains, CLDR plural rules, date and time formats, and the locale checker used
by `cargo xtask check i18n`. No ICU, no runtime files.

- Format and workflow: [`docs/architecture/i18n.md`](../../docs/architecture/i18n.md).
- Contract: [`SPEC.md`](SPEC.md).
- License: GPL-3.0-or-later.
