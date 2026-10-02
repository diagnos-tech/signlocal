//! Masked_cpf and formatted_cnpj.

use super::*;

fn with_cpf(cpf: &str) -> IcpBrasil {
    IcpBrasil {
        cpf: Some(cpf.to_owned()),
        ..IcpBrasil::default()
    }
}

fn with_cnpj(cnpj: &str) -> IcpBrasil {
    IcpBrasil {
        cnpj: Some(cnpj.to_owned()),
        ..IcpBrasil::default()
    }
}

#[test]
fn masks_the_cpf_like_gov_br() {
    let cases = [
        ("12345678901", "***.456.789-**"),
        ("98765432100", "***.654.321-**"),
        ("00000000001", "***.000.000-**"),
        ("11111111111", "***.111.111-**"),
        ("99988877766", "***.888.777-**"),
    ];
    for (cpf, masked) in cases {
        assert_eq!(with_cpf(cpf).masked_cpf().as_deref(), Some(masked), "{cpf}");
    }
}

#[test]
fn masked_cpf_is_none_without_a_cpf() {
    assert_eq!(IcpBrasil::default().masked_cpf(), None);
    assert_eq!(with_cnpj(CNPJ).masked_cpf(), None);
}

#[test]
fn masks_the_cpf_read_from_a_certificate() {
    assert_eq!(
        icp("icp-pf-a3").masked_cpf().as_deref(),
        Some("***.456.789-**")
    );
    assert_eq!(
        icp("icp-pj-a1").masked_cpf().as_deref(),
        Some("***.654.321-**")
    );
    assert_eq!(icp("icp-pj-no-responsible").masked_cpf(), None);
}

#[test]
fn masked_cpf_never_shows_the_first_or_last_digits() {
    let masked = with_cpf("12345678901").masked_cpf().unwrap();
    for hidden in ["123", "01", "12345678901", "456.789-01"] {
        assert!(!masked.contains(hidden), "{masked} leaks {hidden}");
    }
}

#[test]
fn formats_the_cnpj_with_dots_slash_and_dash() {
    let cases = [
        ("12345678000195", "12.345.678/0001-95"),
        ("00000000000191", "00.000.000/0001-91"),
        ("98765432000100", "98.765.432/0001-00"),
        ("11222333000181", "11.222.333/0001-81"),
    ];
    for (cnpj, formatted) in cases {
        assert_eq!(
            with_cnpj(cnpj).formatted_cnpj().as_deref(),
            Some(formatted),
            "{cnpj}"
        );
    }
}

#[test]
fn formatted_cnpj_is_none_without_a_cnpj() {
    assert_eq!(IcpBrasil::default().formatted_cnpj(), None);
    assert_eq!(with_cpf(CPF).formatted_cnpj(), None);
}

#[test]
fn formats_the_cnpj_read_from_a_certificate() {
    assert_eq!(
        icp("icp-pj-a1").formatted_cnpj().as_deref(),
        Some("12.345.678/0001-95")
    );
    assert_eq!(icp("icp-pf-a3").formatted_cnpj(), None);
}

#[test]
fn formatting_refuses_public_fields_that_are_not_all_digits() {
    // The fields are public, so a caller can put anything in them; only
    // exactly 11 (CPF) or 14 (CNPJ) ASCII digits are formatted.
    let odd = [
        "",
        "1",
        "123",
        "abcdefghijk",
        "12345678901234567890",
        "ééééééééééé",
        "ééééééééééééééé",
        "12345678é01",
        "1234567890\u{0}",
        "🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀",
    ];
    for text in odd {
        assert_eq!(with_cpf(text).masked_cpf(), None, "{text:?}");
        assert_eq!(with_cnpj(text).formatted_cnpj(), None, "{text:?}");
    }
}
