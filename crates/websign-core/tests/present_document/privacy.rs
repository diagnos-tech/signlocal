//! The full CPF and national identifier never leak.

use super::*;

/// Whether `text` contains a run of `len` or more ASCII digits.
fn has_digit_run(text: &str, len: usize) -> bool {
    let mut run = 0;
    for c in text.chars() {
        run = if c.is_ascii_digit() { run + 1 } else { 0 };
        if run >= len {
            return true;
        }
    }
    false
}

#[test]
fn the_full_cpf_never_appears_in_the_label() {
    let mut infos: Vec<(String, CertInfo)> = vec![
        (
            "cpf".into(),
            icp(Some("12345678909"), Some("12345678000190")),
        ),
        ("cpf-only".into(), icp(Some("98765432100"), None)),
    ];
    // Every fixture that parses (the `bad-ext-*` ones are malformed on purpose).
    for name in cert_names() {
        if let Ok(parsed) = CertInfo::from_der(&cert(&name)) {
            infos.push((name, parsed));
        }
    }
    assert!(infos.len() > 100);
    for (name, info) in infos {
        let Some(label) = display_document(&info) else {
            continue;
        };
        let text = format!("{label:?}");
        if let Some(cpf) = info.icp_brasil.as_ref().and_then(|i| i.cpf.as_deref()) {
            assert!(!text.contains(cpf), "{name}: {text}");
        }
        if matches!(label, DocumentLabel::Cpf { .. }) {
            assert!(!has_digit_run(&text, 7), "{name}: {text}");
        }
    }
}

#[test]
fn the_full_national_identifier_never_appears_in_the_label() {
    let text = format!("{:?}", display_document(&with_serial("IDCPT-12345123")));
    assert!(!text.contains("12345123"));
    assert!(!text.contains("IDCPT"));
}

#[test]
fn never_panics_on_odd_fields() {
    let long = "A".repeat(10_000);
    let odd = [
        "\u{0}",
        "•••••",
        "IDCPT-\u{1f4a9}\u{1f4a9}\u{1f4a9}",
        "IDCPT-é",
        "IDCPT-ééé",
        long.as_str(),
    ];
    for value in odd {
        let _ = display_document(&with_serial(value));
        let _ = display_document(&icp(Some(value), Some(value)));
    }
}

#[test]
fn the_debug_form_of_a_certificate_masks_its_document_numbers() {
    let person = format!("{:?}", info("icp-pf-a3"));
    assert!(!person.contains("12345678901"), "{person}");
    assert!(person.contains("***.456.789-**"), "{person}");
    let company = format!("{:?}", info("icp-pj-a1"));
    assert!(!company.contains("98765432100"), "{company}");
    let european = format!("{:?}", info("dn-pii"));
    assert!(!european.contains("12345123"), "{european}");
}
