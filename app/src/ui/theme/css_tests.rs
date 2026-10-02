//! `design/tokens.css` is the single source: every `--ws-*` token must equal
//! its Rust counterpart, and every token must have one (or be listed as
//! CSS-only below with the reason).

use super::css_expected::{color_table, scalar_table};
use super::css_parse::{Block, blocks, color as parse_color};
use super::tokens::{self, Colors, IDENTICON};

const CSS: &str = include_str!("../../../../design/tokens.css");

/// Selectors of the blocks the test reads.
const BASE: &str = ":root";
const DARK_PINNED: &str = ":root[data-theme=\"dark\"]";
const DARK_SYSTEM: &str = "@media (prefers-color-scheme: dark) > :root:not([data-theme=\"light\"])";
const REDUCED_MOTION: &str = "@media (prefers-reduced-motion: reduce) > :root";

/// Tokens only the web uses: font stacks (egui embeds its fonts).
const CSS_ONLY: [&str; 3] = ["font-ui", "font-mono", "font-popup"];

fn assert_colors(block: &Block, colors: &Colors, which: &str) {
    for (name, expected) in color_table(colors) {
        let css = block
            .get(name)
            .unwrap_or_else(|| panic!("{which}: --ws-{name} missing"));
        assert_eq!(parse_color(css), expected, "{which}: --ws-{name}");
    }
}

#[test]
fn light_colors_match_the_base_block() {
    assert_colors(&blocks(CSS)[BASE], &tokens::light(), "light");
}

#[test]
fn dark_colors_match_both_dark_blocks() {
    let all = blocks(CSS);
    assert_colors(&all[DARK_PINNED], &tokens::dark(), "dark (pinned)");
    assert_eq!(
        all[DARK_SYSTEM], all[DARK_PINNED],
        "the two dark blocks differ"
    );
}

#[test]
fn identicon_palette_matches() {
    let base = &blocks(CSS)[BASE];
    for (index, expected) in IDENTICON.iter().enumerate() {
        assert_eq!(
            parse_color(&base[&format!("id-{index}")]),
            *expected,
            "id-{index}"
        );
    }
}

#[test]
fn sizes_type_and_motion_match() {
    let base = &blocks(CSS)[BASE];
    for (name, expected) in scalar_table() {
        assert_eq!(base.get(&name), Some(&expected), "--ws-{name}");
    }
}

#[test]
fn every_css_token_has_a_rust_counterpart() {
    let known: Vec<String> = color_table(&tokens::light())
        .into_iter()
        .map(|(name, _)| name.to_owned())
        .chain((0..IDENTICON.len()).map(|i| format!("id-{i}")))
        .chain(scalar_table().into_iter().map(|(name, _)| name))
        .chain(CSS_ONLY.iter().map(|name| (*name).to_owned()))
        .collect();
    for (selector, block) in blocks(CSS) {
        for name in block.keys() {
            assert!(
                known.contains(name),
                "--ws-{name} in `{selector}` has no Rust constant"
            );
        }
    }
}

#[test]
fn reduced_motion_zeroes_animations_only() {
    let reduced = &blocks(CSS)[REDUCED_MOTION];
    let names: Vec<&str> = reduced.keys().map(String::as_str).collect();
    assert_eq!(names, ["motion-base", "motion-fast", "motion-slow"]);
    assert!(reduced.values().all(|value| value == "0ms"));
}
