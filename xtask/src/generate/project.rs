//! `project.toml` → `project.ts` for the packages that name the product.
//!
//! The SDK gets only what its install links need (store IDs, homepage), at
//! `sdk/src/project.ts` where its modules import it: page code never needs
//! the native host or the development extension ID.

use std::path::Path;

use super::text::ts_header;
use crate::fsutil::read_text;
use crate::plan::Plan;

/// Biome's line width; a longer `export const` wraps after `=` like its
/// formatter would, so the file needs no formatting pass.
const LINE_WIDTH: usize = 100;

/// One exported constant and where `project.toml` keeps its value.
struct Constant {
    name: &'static str,
    table: &'static str,
    key: &'static str,
    doc: &'static str,
}

const fn constant(
    name: &'static str,
    table: &'static str,
    key: &'static str,
    doc: &'static str,
) -> Constant {
    Constant {
        name,
        table,
        key,
        doc,
    }
}

const CONSTANTS: [Constant; 10] = [
    constant(
        "PRODUCT_NAME",
        "product",
        "name",
        "Product name shown to people.",
    ),
    constant(
        "NATIVE_HOST",
        "ids",
        "native_host",
        "Native messaging host name.",
    ),
    constant(
        "FIREFOX_ID",
        "extension",
        "firefox_id",
        "Firefox add-on ID.",
    ),
    constant(
        "EXTENSION_DEV_ID",
        "extension",
        "dev_id",
        "Chromium ID of the unpacked development build.",
    ),
    constant(
        "CHROME_WEB_STORE_ID",
        "extension",
        "chrome_web_store_id",
        "Chrome Web Store item ID; empty until published.",
    ),
    constant(
        "EDGE_ADDONS_ID",
        "extension",
        "edge_addons_id",
        "Edge Add-ons product ID; empty until published.",
    ),
    constant(
        "FIREFOX_AMO_SLUG",
        "extension",
        "firefox_amo_slug",
        "addons.mozilla.org listing slug; empty until published.",
    ),
    constant(
        "MIN_APP_VERSION",
        "extension",
        "min_app_version",
        "Oldest app version the extension accepts.",
    ),
    constant(
        "HOMEPAGE",
        "product",
        "homepage",
        "The project's site; its download page lists every way to install.",
    ),
    constant(
        "DEV_KEY",
        "extension",
        "dev_key",
        "Public key that pins the development extension ID.",
    ),
];

/// Output file → the constants it exports (`None`: all of them).
const TARGETS: [(&str, Option<&[&str]>); 3] = [
    ("extension/src/generated/project.ts", None),
    ("clients/node/src/generated/project.ts", None),
    (
        "sdk/src/project.ts",
        Some(&[
            "CHROME_WEB_STORE_ID",
            "EDGE_ADDONS_ID",
            "FIREFOX_AMO_SLUG",
            "HOMEPAGE",
        ]),
    ),
];

pub fn plan(root: &Path) -> Result<Plan, String> {
    let table: toml::Table = read_text(&root.join("project.toml"))?
        .parse()
        .map_err(|e| format!("project.toml is not valid TOML: {e}"))?;
    let mut plan = Plan::default();
    for (path, only) in TARGETS {
        plan.add(path, render(&table, only)?);
    }
    Ok(plan)
}

fn render(table: &toml::Table, only: Option<&[&str]>) -> Result<String, String> {
    let mut out = ts_header("project.toml");
    let wanted = CONSTANTS
        .iter()
        .filter(|c| only.is_none_or(|names| names.contains(&c.name)));
    for Constant {
        name,
        table: section,
        key,
        doc,
    } in wanted
    {
        let value = table
            .get(*section)
            .and_then(|s| s.get(*key))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| format!("project.toml: missing string `[{section}] {key}`"))?;
        let literal = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let line = format!("export const {name} = {literal};");
        out.push_str(&format!("\n/** {doc} */\n"));
        if line.len() > LINE_WIDTH {
            out.push_str(&format!("export const {name} =\n  {literal};\n"));
        } else {
            out.push_str(&line);
            out.push('\n');
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[product]
name = "N"
homepage = "https://h/"
[ids]
native_host = "a.b"
[extension]
firefox_id = "f@x"
dev_id = "abc"
chrome_web_store_id = ""
edge_addons_id = ""
firefox_amo_slug = ""
min_app_version = "0.1.0"
dev_key = "K"
"#;

    fn table(text: &str) -> toml::Table {
        text.parse().unwrap()
    }

    #[test]
    fn renders_every_constant_with_its_doc() {
        let out = render(&table(SAMPLE), None).unwrap();
        assert!(out.contains("\nexport const NATIVE_HOST = \"a.b\";\n"));
        assert!(out.contains("*/\nexport const CHROME_WEB_STORE_ID = \"\";\n"));
    }

    #[test]
    fn a_subset_leaves_the_other_ids_out() {
        let out = render(&table(SAMPLE), Some(&["HOMEPAGE"])).unwrap();
        assert!(out.contains("HOMEPAGE = \"https://h/\""));
        assert!(!out.contains("NATIVE_HOST") && !out.contains("DEV_KEY"));
    }

    #[test]
    fn long_values_wrap_after_the_equals_sign() {
        let long = SAMPLE.replace("\"K\"", &format!("\"{}\"", "k".repeat(120)));
        let out = render(&table(&long), None).unwrap();
        assert!(out.contains("DEV_KEY =\n  \"kkkk"));
    }

    #[test]
    fn a_missing_key_is_named() {
        let error = render(&table("[product]\nname = \"N\""), None).unwrap_err();
        assert!(error.contains("[ids] native_host"), "{error}");
    }
}
