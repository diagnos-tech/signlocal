# site/src/i18n

The seven languages: English at the root, the others under /pt-br/, /pt-pt/, /es/, /fr/, /it/ and /de/.

- `catalog.ts` — loads the seven TOML files at build time
- `choose.ts` — picks a language from ?lang, the saved choice and the browser's languages
- `codes.ts` — the language codes (tiny: the boot script uses it)
- `locales.ts` — folders, names and Open Graph codes of the languages
- `messages.ts` — flattens the TOML, looks messages up, checks placeholders
- `messages/` — the texts, one TOML file per language
- `rich.ts` — escapes a message and turns **bold**, `code`, __ink__ and links into HTML
