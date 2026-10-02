//! A reader for the small CSS subset `design/tokens.css` uses: comments,
//! nested blocks (`@media { :root { … } }`) and `--ws-*: value;`
//! declarations. Test-only: the app never reads CSS at run time.

use std::collections::BTreeMap;

use egui::Color32;

/// Declarations of one block: token name without `--ws-` → value.
pub type Block = BTreeMap<String, String>;

/// Every block that declares tokens, keyed by its selector path: the
/// enclosing selectors joined with `" > "`, e.g.
/// `@media (prefers-reduced-motion: reduce) > :root`.
pub fn blocks(css: &str) -> BTreeMap<String, Block> {
    let text = strip_comments(css);
    let mut result: BTreeMap<String, Block> = BTreeMap::new();
    let mut stack: Vec<String> = Vec::new();
    let mut pending = String::new();
    for c in text.chars() {
        match c {
            '{' => stack.push(std::mem::take(&mut pending).trim().to_owned()),
            '}' => {
                stack.pop();
                pending.clear();
            }
            ';' => {
                let declaration = std::mem::take(&mut pending);
                if let Some((name, value)) = declaration.trim().split_once(':')
                    && let Some(token) = name.trim().strip_prefix("--ws-")
                {
                    result
                        .entry(stack.join(" > "))
                        .or_default()
                        .insert(token.to_owned(), value.trim().to_owned());
                }
            }
            _ => pending.push(c),
        }
    }
    result
}

fn strip_comments(css: &str) -> String {
    let mut text = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        text.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    text.push_str(rest);
    text
}

/// `#RRGGBB` or `#RRGGBBAA` (unmultiplied alpha, as CSS means it).
pub fn color(value: &str) -> Color32 {
    let hex = value.strip_prefix('#').expect("a #hex color");
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex digits");
    let alpha = if hex.len() == 8 { byte(6) } else { 255 };
    Color32::from_rgba_unmultiplied(byte(0), byte(2), byte(4), alpha)
}

#[test]
fn nests_selectors_and_skips_comments() {
    let css = "/* a; b { */ :root { --ws-x: 1px; color: red; } @media m { :root { --ws-x: 0; } }";
    let all = blocks(css);
    assert_eq!(all[":root"]["x"], "1px");
    assert_eq!(all["@media m > :root"]["x"], "0");
    assert_eq!(all.len(), 2);
}
