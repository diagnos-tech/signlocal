//! `[site.errors]` of every locale → `sdk/src/messages.gen.ts`.
//!
//! Exported only from `@websign/sdk/messages`, so sites that bring their own
//! texts do not ship these. `MESSAGES` is typed with the protocol's
//! `ErrorCode`, so a misspelled code in `i18n/` fails the SDK typecheck;
//! codes a site never shows (`ClientOutdated`, …) simply have no entry.

use std::path::Path;

use super::text::{pascal_case, ts_header};
use crate::i18n_files::{Locale, load_locales};
use crate::plan::Plan;

pub fn plan(root: &Path) -> Result<Plan, String> {
    let mut plan = Plan::default();
    plan.add("sdk/src/messages.gen.ts", render(&load_locales(root)?)?);
    Ok(plan)
}

/// Biome's line width, for the `MessageLocale` union.
const LINE_WIDTH: usize = 100;

const PREAMBLE: &str = "import type { ErrorCode } from \"./generated/index.js\";

/** A title and a sentence; `{name}` placeholders are left for the caller. */
export interface MessageText {
  readonly title: string;
  readonly body: string;
}

/** The texts of one locale; codes a site never shows have none. */
export type LocaleMessages = Readonly<Partial<Record<ErrorCode, MessageText>>>;
";

fn render(locales: &[Locale]) -> Result<String, String> {
    let mut out = ts_header("i18n/*.toml [site.errors]");
    out.push('\n');
    out.push_str(PREAMBLE);
    out.push_str(&locale_union(locales));
    out.push_str(
        "\n/** Title and body per error code, per locale. */\n\
         export const MESSAGES: Readonly<Record<MessageLocale, LocaleMessages>> = {\n",
    );
    for locale in locales {
        let errors = locale
            .table
            .get("site")
            .and_then(|site| site.get("errors"))
            .and_then(toml::Value::as_table)
            .ok_or_else(|| format!("i18n/{}.toml: missing [site.errors]", locale.name))?;
        out.push_str(&format!("  {}: {{\n", property(&locale.name)));
        for (code, texts) in errors {
            out.push_str(&format!("    {}: {{\n", pascal_case(code)));
            for field in ["title", "body"] {
                let text = texts
                    .get(field)
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| {
                        format!(
                            "i18n/{}.toml: site.errors.{code}.{field} is not a string",
                            locale.name
                        )
                    })?;
                let literal = serde_json::to_string(text).map_err(|e| e.to_string())?;
                out.push_str(&format!("      {field}: {literal},\n"));
            }
            out.push_str("    },\n");
        }
        out.push_str("  },\n");
    }
    out.push_str("};\n");
    Ok(out)
}

/// `export type MessageLocale = "en" | …;`, wrapped one member per line
/// when too long, as Biome formats it.
fn locale_union(locales: &[Locale]) -> String {
    let names: Vec<String> = locales.iter().map(|l| format!("\"{}\"", l.name)).collect();
    let doc = "\n/** Locales with texts, in the order `i18n.md` lists them. */\n";
    let line = format!("export type MessageLocale = {};", names.join(" | "));
    if line.len() <= LINE_WIDTH {
        return format!("{doc}{line}\n");
    }
    let members: String = names.iter().map(|n| format!("\n  | {n}")).collect();
    format!("{doc}export type MessageLocale ={members};\n")
}

/// An object key: bare when it is an identifier, quoted otherwise (`pt-BR`),
/// which is the form Biome's formatter keeps.
fn property(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        name.to_owned()
    } else {
        format!("\"{name}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(name: &str, toml: &str) -> Locale {
        Locale {
            name: name.into(),
            table: toml.parse().unwrap(),
        }
    }

    #[test]
    fn renders_codes_in_pascal_case_and_quotes_odd_locale_names() {
        let toml = "[site.errors.user_cancelled]\ntitle = \"T\"\nbody = \"B \\\"q\\\"\"\n";
        let out = render(&[locale("en", toml), locale("pt-BR", toml)]).unwrap();
        assert!(out.contains("  en: {\n    UserCancelled: {\n      title: \"T\",\n"));
        assert!(out.contains("  \"pt-BR\": {"));
        assert!(out.contains("export type MessageLocale = \"en\" | \"pt-BR\";\n"));
        assert!(out.contains("body: \"B \\\"q\\\"\","));
    }

    #[test]
    fn a_long_locale_union_wraps_one_member_per_line() {
        let many: Vec<Locale> = (0..12)
            .map(|i| locale(&format!("locale-{i}"), ""))
            .collect();
        assert!(locale_union(&many).contains("MessageLocale =\n  | \"locale-0\"\n  | "));
    }

    #[test]
    fn a_locale_without_site_errors_is_named() {
        let error = render(&[locale("fr", "[meta]\nx = \"y\"")]).unwrap_err();
        assert!(error.contains("i18n/fr.toml"), "{error}");
    }
}
