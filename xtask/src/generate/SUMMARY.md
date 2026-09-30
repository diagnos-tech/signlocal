# xtask/src/generate

- `locales.rs` — extension `_locales/<xx_YY>/messages.json` from the popup, store and extension texts
- `messages.rs` — SDK error texts (`messages.gen.ts`) from `[site.errors]`
- `mod.rs` — `cargo xtask gen`: runs the generators into one plan
- `project.rs` — `project.ts` constants from project.toml
- `text.rs` — generated-file headers, SUMMARY.md rendering and case conversion
- `ts.rs` — protocol types exported by ts-rs into the SDK, extension and Node client
