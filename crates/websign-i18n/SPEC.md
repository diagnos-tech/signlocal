# websign-i18n — specification

Rules and vectors for the localization engine. Format and workflow:
[`docs/architecture/i18n.md`](../../docs/architecture/i18n.md). The key
generator (`build.rs`) is implemented; everything under `src/` with
`todo!()` is to be implemented blind.

## 1. Keys (implemented by `build.rs`)

- Every string leaf of `i18n/en.toml` becomes `k::<PATH>: Key` (dots and
  dashes → `_`, uppercase): `confirm.window_title` → `k::CONFIRM_WINDOW_TITLE`.
- A table whose keys are all CLDR categories and contains `other` becomes one
  `PluralKey` (`cert.expires_in` → `k::CERT_EXPIRES_IN`).
- `ALL_KEYS` lists every path in file order; `SOURCES` embeds the 7 files.

## 2. `Locale`

`match_tag(tag)`: case-insensitive; `_` and `-` equivalent; encoding suffixes
(`.UTF-8`) and modifiers (`@euro`) ignored.

| Input | Result |
|---|---|
| `en`, `en-US`, `en_GB.UTF-8` | En |
| `pt-BR`, `pt_br`, `pt` | PtBr |
| `pt-PT`, `pt_PT@euro`, `pt-AO`, `pt-MZ` | PtPt |
| `es`, `es-MX`, `es_ES` | Es |
| `fr`, `fr-CA`, `fr_BE` | Fr |
| `it`, `it-CH` | It |
| `de`, `de-AT`, `de_CH` | De |
| `ja`, `zh-CN`, `""`, `C`, `POSIX` | `None` |

`from_system()`: the first of `sys_locale::get_locales()` that matches, else En.

`fallback_chain()`: En → `[En]`; PtBr → `[PtBr, PtPt, En]`; PtPt →
`[PtPt, PtBr, En]`; Es/Fr/It/De → `[self, En]`.

## 3. `Catalog`

- `new(locale)` parses the embedded sources of the chain once. A source that
  fails to parse is skipped (logged with `log::error!`), never panics.
- `tr(key)`: the template from the first locale of the chain that has the
  key; if none has it, the template is the key path itself (visible, caught by
  review).
- `plural(key, count)`: category from `plural::category(locale_found, count)`;
  if that category is missing in the table, `other`; `{count}` is filled
  with the decimal integer.

## 4. `Message` rendering

- `{name}` with `name` in `[a-z_]+` is replaced by the last `arg` given for
  it; without an argument it stays as `{name}`.
- Any other `{`/`}` is literal (`"{ x }"`, `"{Name}"`, `"{"` stay).
- Arguments without a placeholder are ignored.
- Vectors: `"Sign for {site} — WebeSign"` + site=`a.b` → `"Sign for a.b —
  WebeSign"`; `"{a}{a}"` + a=`x` → `"xx"`; `"{missing}"` → `"{missing}"`;
  `"{{x}}"` + x=`1` → `"{1}"`.

## 5. Plural rules (CLDR cardinal, integers only)

| Locale | one | many | other |
|---|---|---|---|
| en, de, it, es | n = 1 | es, it: n ≠ 0 and n mod 1 000 000 = 0 | rest |
| pt-PT | n = 1 | n ≠ 0 and n mod 1 000 000 = 0 | rest |
| pt-BR | n = 0 or 1 | n ≠ 0 and n mod 1 000 000 = 0 | rest |
| fr | n = 0 or 1 | n ≠ 0 and n mod 1 000 000 = 0 | rest |

German and English have no `many`. Negative numbers use their absolute
value. Vectors: en 0→other, 1→one, 2→other; pt-BR 0→one, 1→one, 2→other,
1 000 000→many; fr 1→one, 2→other; de 1 000 000→other; es 1 000 000→many.

## 6. Dates and times

`format_date`: en `14 Mar 2027` (English 3-letter month); pt-BR, pt-PT, es,
fr, it `14/03/2027`; de `14.03.2027`. Day and month zero-padded in numeric
forms; day not padded in en (`4 Mar 2027`). `format_time`: `HH:MM`, 24-hour,
zero-padded, every locale.

## 7. `check::check_locale(reference, candidate)`

Problems, in this order of detection, all reported (not just the first):

1. `Syntax` — candidate not valid TOML, or a value that is neither a string
   nor a table.
2. `Missing` — reference key absent in candidate.
3. `Extra` — candidate key absent in reference.
4. `Placeholders` — for strings, the set of `{name}` differs; for plurals,
   every category's set must equal the reference `other`'s set.
5. `Plural` — a plural without `other`, or with a category outside CLDR.
6. `Extension` — a plural under `[popup]`, `[store]` or `[extension]`, or
   `extension.description` longer than 132 characters.

Keys are compared by full dotted path; a table-vs-string mismatch is one
`Missing` + one `Extra`.
