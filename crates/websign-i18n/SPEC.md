# websign-i18n — specification

Rules and vectors for the localization engine. Format and workflow:
[`docs/architecture/i18n.md`](../../docs/architecture/i18n.md).

## 1. Keys (implemented by `build.rs`)

- Every string leaf of `i18n/en.toml` becomes `k::<PATH>: Key` (dots and
  dashes → `_`, uppercase): `confirm.window_title` → `k::CONFIRM_WINDOW_TITLE`.
- A table whose keys are all CLDR categories and contains `other` becomes one
  `PluralKey` (`cert.expires_in` → `k::CERT_EXPIRES_IN`).
- A table with any other key (`several`, a nested table) is a section: its
  string leaves are ordinary keys.
- `k`, `ALL_KEYS` (every path in file order) and `SOURCES` (`(tag, TOML)` of
  the 7 files, `en` first) are exported at the crate root.

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
  fails to parse is skipped (logged with `log::error!`: locale tag and parse
  error), never panics.
- `from_sources(locale, &[(Locale, &str)])` (`#[doc(hidden)]`) builds the
  same catalog over explicit sources in lookup order; `new` uses it, and
  tests use it to exercise fallback, which shipped files never need.
- `tr(key)`: the first string found at the key's path along the chain (a
  table there does not count); if none, the template is the key path itself
  (visible, caught by review).
- `plural(key, count)`: the first table found at the path along the chain (a
  string there does not count). Category from
  `plural::category(locale_that_had_the_table, count)`; if that category is
  missing in the table, `other`; if `other` is missing too, the key path.
  `{count}` is filled with the decimal integer, sign included (`-5`).

## 4. `Message` rendering

- `{name}` with `name` in `[a-z_]+` is replaced by the last `arg` given for
  it; without an argument it stays as `{name}`.
- Any other `{`/`}` is literal (`"{ x }"`, `"{Name}"`, `"{"` stay).
- Arguments without a placeholder are ignored.
- Substitution is one pass over the template: an argument value is inserted
  verbatim and never scanned for placeholders (`"{a} {b}"` + a=`{b}`, b=`x`
  → `"{b} x"`), so user-controlled values cannot inject other arguments.
  Values are not escaped; the UI layer escapes for its medium.
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
value (`i64::MIN` included, no overflow). Only the six CLDR categories exist
(`plural::PluralCategory`). Vectors: en 0→other, 1→one, 2→other; pt-BR 0→one, 1→one, 2→other,
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
   every category's set must equal the reference `other`'s set (one problem
   per differing form). `expected`/`found` are sorted, deduplicated names.
5. `Plural` — a plural without `other`, or with a category outside CLDR
   (`several`). A candidate table is a plural when the reference has a plural
   at that path, or when it has `other` and only CLDR keys; so a renamed or
   invented category is a `Plural` problem, not `Missing`/`Extra`.
6. `Extension` — a plural under `[popup]`, `[store]` or `[extension]`, or
   `extension.description` longer than 132 characters (Unicode scalar
   values, not bytes). Checked on the candidate, so `check_locale(en, en)`
   also catches them in the reference.

A `Syntax` problem in either file stops the comparison: only the `Syntax`
problems are returned. Keys are compared by full dotted path; a
table-vs-string mismatch is one `Missing` + one `Extra`. Hostile input never
panics.
