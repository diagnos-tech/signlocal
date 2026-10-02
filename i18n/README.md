# i18n

Every text a person reads, one TOML file per locale: `en.toml` (the
reference: every key, with translator notes), `pt-BR`, `pt-PT`, `es`, `fr`,
`it`, `de`. The other files have exactly the keys and `{placeholders}` of
`en.toml`. Format and rules: [`docs/architecture/i18n.md`](../docs/architecture/i18n.md).

Used by:

- the app, through [`crates/websign-i18n`](../crates/websign-i18n/) (embedded
  at build time; a misspelled key does not compile);
- the extension: `cargo xtask gen` writes `[popup]`, `[store]` and
  `[extension]` into `extension/public/_locales/<xx_YY>/messages.json`.
  `{name}` becomes `$name$`, and the substitutions are numbered in the
  alphabetical order of the placeholder names, so the popup passes
  `getMessage("popup_footer_versions", [app, ext])` for every language;
- the SDK: `[site.errors]` becomes `sdk/src/messages.gen.ts`.

After editing: `cargo xtask check i18n` (keys, placeholders, plurals, store
limits), then `cargo xtask gen` and commit the generated files
(`cargo xtask check generated` fails otherwise).

## License

The TOML files here, like `project.toml`, are licensed GPL-3.0-or-later **or**
Apache-2.0, at your option (see [`LICENSE`](../LICENSE)). The Apache-2.0 SDK
ships text generated from them (`sdk/src/messages.gen.ts`), so a GPL-only
source would pull the SDK under the GPL. Contributing a translation means
offering it under both licenses. This README and the tooling stay
GPL-3.0-or-later.
