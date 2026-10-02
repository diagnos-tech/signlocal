//! SPEC §6.1 and `display_name` (§6.5): `DistinguishedName`.

mod common;

use common::{blank_info, dn, fixture_fingerprint, info, leaf_dn, root_dn};
use websign_core::{CertInfo, DistinguishedName, Fingerprint, IcpBrasil};

#[test]
fn reads_subject_and_issuer_of_an_ordinary_certificate() {
    let info = info("rsa2048");
    assert_eq!(info.subject, leaf_dn("rsa2048"));
    assert_eq!(info.issuer, root_dn());
}

#[test]
fn a_self_signed_root_has_equal_subject_and_issuer() {
    let root = info("ca");
    assert_eq!(root.subject, root_dn());
    assert_eq!(root.issuer, root.subject);
}

#[test]
fn keeps_organizational_units_in_certificate_order() {
    assert_eq!(
        info("p256").subject.organizational_units,
        ["Unit A", "Unit B"]
    );
    assert_eq!(
        info("dn-duplicates").subject.organizational_units,
        ["A", "B", "C"]
    );
}

#[test]
fn first_occurrence_wins_for_single_valued_attributes() {
    let subject = info("dn-duplicates").subject;
    assert_eq!(subject.country.as_deref(), Some("BR"));
    assert_eq!(subject.organization.as_deref(), Some("First"));
    assert_eq!(subject.common_name.as_deref(), Some("One"));
}

#[test]
fn reads_attributes_of_a_multi_valued_rdn() {
    assert_eq!(
        info("dn-multivalue-rdn").subject,
        dn(Some("Multi"), Some("Org"), &["Unit"], Some("BR"))
    );
}

#[test]
fn decodes_string_types_as_the_spec_says() {
    let cases = [
        (
            "dn-utf8",
            dn(
                Some("JOSÉ AÇAÍ DA SILVA"),
                Some("Açougue & Cia Ltda"),
                &["Divisão São João"],
                Some("BR"),
            ),
        ),
        (
            "dn-printable",
            dn(
                Some("PRINTABLE NAME 123"),
                Some("Printable Org"),
                &["Printable Unit"],
                Some("BR"),
            ),
        ),
        (
            "dn-ia5",
            dn(
                Some("ia5.name.example.com"),
                Some("ia5.org.example"),
                &["ia5.unit"],
                Some("BR"),
            ),
        ),
        // TeletexString is Latin-1: 0xC9 is É, 0xE7 is ç, 0xE3 is ã.
        (
            "dn-teletex",
            dn(
                Some("JOSÉ TELETEX"),
                Some("Ação Ltda"),
                &["Divisão"],
                Some("BR"),
            ),
        ),
        // BMPString is UTF-16BE; Ω (U+03A9) is outside Latin-1.
        (
            "dn-bmp",
            dn(
                Some("JOSÉ Ω BMP"),
                Some("Êxito Ltda"),
                &["Divisão"],
                Some("BR"),
            ),
        ),
    ];
    for (name, expected) in cases {
        assert_eq!(info(name).subject, expected, "{name}");
    }
}

#[test]
fn ignores_attributes_of_other_string_types_and_keeps_the_rest() {
    // NumericString and UniversalString are not among the types read: only
    // that attribute is dropped. `der` cannot even skip a UniversalString, so
    // this also proves the parser does not choke on one.
    assert_eq!(
        info("dn-numeric").subject,
        dn(None, Some("Numeric Org"), &["First", "Third"], Some("BR"))
    );
    assert_eq!(
        info("dn-universal").subject,
        dn(None, Some("Universal Org"), &["Unit"], Some("BR"))
    );
}

#[test]
fn missing_attributes_stay_none() {
    assert_eq!(info("dn-empty").subject, DistinguishedName::default());
    assert_eq!(
        info("dn-country-only").subject,
        dn(None, None, &[], Some("BR"))
    );
    assert_eq!(
        info("dn-org-only").subject,
        dn(None, Some("Only Org Name"), &[], None)
    );
}

#[test]
fn default_distinguished_name_is_empty() {
    let empty = DistinguishedName::default();
    assert_eq!(empty, dn(None, None, &[], None));
}

fn named(cn: Option<&str>, org: Option<&str>) -> DistinguishedName {
    dn(cn, org, &["Some Unit"], Some("BR"))
}

#[test]
fn display_name_prefers_the_icp_brasil_holder() {
    let info = CertInfo {
        subject: named(Some("CN Name:123"), Some("Org")),
        icp_brasil: Some(IcpBrasil {
            holder_name: Some("HOLDER".to_owned()),
            ..IcpBrasil::default()
        }),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "HOLDER");
}

#[test]
fn display_name_falls_back_to_the_common_name() {
    let icp_without_holder = Some(IcpBrasil::default());
    for icp_brasil in [None, icp_without_holder] {
        let info = CertInfo {
            subject: named(Some("Common Name"), Some("Org")),
            icp_brasil,
            ..blank_info()
        };
        assert_eq!(info.display_name(), "Common Name");
    }
}

#[test]
fn display_name_falls_back_to_the_organization() {
    let info = CertInfo {
        subject: named(None, Some("Only Org")),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "Only Org");
}

#[test]
fn display_name_last_resort_is_the_first_16_hex_digits_of_the_fingerprint() {
    let info = CertInfo {
        subject: named(None, None),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "abababababababab");

    let bytes: [u8; 32] = std::array::from_fn(|i| i as u8 * 7 + 1);
    let fingerprint = Fingerprint::from_bytes(bytes);
    let info = CertInfo {
        fingerprint,
        subject: DistinguishedName::default(),
        ..blank_info()
    };
    assert_eq!(info.display_name(), fingerprint.to_hex()[..16]);
    assert_eq!(info.display_name().len(), 16);
}

#[test]
fn display_name_skips_blank_candidates() {
    let blank_holder = CertInfo {
        subject: named(Some("Common Name"), Some("Org")),
        icp_brasil: Some(IcpBrasil {
            holder_name: Some(String::new()),
            ..IcpBrasil::default()
        }),
        ..blank_info()
    };
    assert_eq!(blank_holder.display_name(), "Common Name");
    let blank_cn = CertInfo {
        subject: named(Some(" \t "), Some("Org")),
        ..blank_info()
    };
    assert_eq!(blank_cn.display_name(), "Org");
    let all_blank = CertInfo {
        subject: named(Some(""), Some(" ")),
        ..blank_info()
    };
    assert_eq!(all_blank.display_name(), "abababababababab");
}

#[test]
fn display_name_does_not_use_units_or_country() {
    let info = CertInfo {
        subject: dn(None, None, &["Unit"], Some("BR")),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "abababababababab");
}

#[test]
fn display_name_on_the_fixtures() {
    assert_eq!(info("rsa2048").display_name(), "Fixture rsa2048");
    assert_eq!(info("icp-pf-a3").display_name(), "ANA BEATRIZ SOUZA");
    assert_eq!(info("dn-org-only").display_name(), "Only Org Name");
    // Not ICP-Brasil, so the ":digits" suffix stays.
    assert_eq!(info("non-icp-upn").display_name(), "PLAIN NAME:12345678901");
    // No CN in the numeric-CN fixture (ignored): the organization is next.
    assert_eq!(info("dn-numeric").display_name(), "Numeric Org");
    for name in ["dn-empty", "dn-country-only"] {
        let info = info(name);
        assert_eq!(
            info.display_name(),
            fixture_fingerprint(name).to_hex()[..16],
            "{name}"
        );
    }
}
