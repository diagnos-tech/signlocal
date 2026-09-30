//! `mask_atr` on every ATR pattern of `devices.json`: each one is a valid ATR,
//! so the interface bytes survive and everything after them is masked.

use websign_ui_model::diagnostics::atr_mask::mask_atr;

/// ATR patterns of `devices.json`, with `..` wildcards filled in as `00`.
fn catalog_atrs() -> Vec<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../devices.json");
    let json = std::fs::read_to_string(path).unwrap_or_default();
    json.lines()
        .map(|line| line.trim().trim_matches(|c| c == '"' || c == ','))
        .filter(|text| text.starts_with("3B") || text.starts_with("3F"))
        .map(|text| text.replace("..", "00"))
        .collect()
}

#[test]
fn every_catalog_atr_keeps_a_prefix_and_masks_the_rest() {
    let atrs = catalog_atrs();
    assert!(
        atrs.len() >= 25,
        "devices.json ATRs not found: {}",
        atrs.len()
    );
    for atr in atrs {
        let masked = mask_atr(&atr);
        assert!(
            masked.starts_with("3B:") || masked.starts_with("3F:"),
            "{atr}: {masked}"
        );
        assert!(masked.contains(".."), "{atr}: {masked}");
        let shown: Vec<&str> = masked.split(':').take_while(|byte| *byte != "..").collect();
        let tail = masked.split(':').skip(shown.len());
        assert!(tail.clone().all(|byte| byte == ".."), "{atr}: {masked}");
        assert_eq!(masked.split(':').count(), atr.len() / 2, "{atr}: {masked}");
        assert!(shown.len() < atr.len() / 2, "{atr}: {masked}");
    }
}

#[test]
fn a_catalog_atr_with_its_last_byte_removed_falls_back_to_the_length() {
    for atr in catalog_atrs() {
        let cut = &atr[..atr.len() - 2];
        assert_eq!(mask_atr(cut), format!("{} bytes", cut.len() / 2), "{cut}");
    }
}
