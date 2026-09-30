# crates/websign-i18n/tests

- `fixtures/` — a small reference locale file for the checker tests
- `catalog.rs` — SPEC §3: `Catalog` over the shipped, embedded locales.
- `check.rs` — SPEC §7: `check_locale` syntax, missing/extra keys and placeholders.
- `check_plural.rs` — SPEC §7: plural and browser-extension rules of `check_locale`.
- `dates.rs` — SPEC §6: `format_date` and `format_time`.
- `fallback.rs` — SPEC §3: fallback along the chain, with injected sources.
- `keys.rs` — SPEC §1: generated `k` constants, `ALL_KEYS` and `SOURCES`.
- `locale.rs` — SPEC §2: `Locale` matching and fallback chains.
- `message.rs` — SPEC §4: placeholder rendering.
- `plural.rs` — SPEC §5: CLDR plural rules.
- `shipped.rs` — the repository's real locale files, checked the way CI does.
