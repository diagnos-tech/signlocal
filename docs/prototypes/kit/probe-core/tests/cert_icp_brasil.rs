//! SPEC §6.3: ICP-Brasil detection, level, holder, CPF and CNPJ.
//!
//! Certificate values come from `tests/fixtures/gen/certs-icp.sh`:
//! CPF 12345678901 (birth 15051990) for the person, CNPJ 12345678000195 for
//! the company, CPF 98765432100 for the company's responsible person and
//! CPF 22222222222 as a second value to tell the otherNames apart.

mod common;

use std::collections::HashSet;

use common::{cert, info};
use probe_core::{CertInfo, IcpBrasil, IcpLevel};

use IcpLevel::{A1, A2, A3, A4, Other, S1, S2, S3, S4, T3, T4};

const CPF: &str = "12345678901";
const CNPJ: &str = "12345678000195";
const RESPONSIBLE_CPF: &str = "98765432100";
const OTHER_CPF: &str = "22222222222";

fn icp(name: &str) -> IcpBrasil {
    info(name)
        .icp_brasil
        .unwrap_or_else(|| panic!("{name} should be recognized as ICP-Brasil"))
}

fn expected(
    level: Option<IcpLevel>,
    holder: Option<&str>,
    cpf: Option<&str>,
    cnpj: Option<&str>,
) -> IcpBrasil {
    IcpBrasil {
        level,
        holder_name: holder.map(str::to_owned),
        cpf: cpf.map(str::to_owned),
        cnpj: cnpj.map(str::to_owned),
    }
}

fn person(cpf: Option<&str>) -> IcpBrasil {
    expected(Some(A3), Some("ANA BEATRIZ SOUZA"), cpf, None)
}

fn company(cnpj: Option<&str>, cpf: Option<&str>) -> IcpBrasil {
    expected(Some(A1), Some("EMPRESA TESTE LTDA"), cpf, cnpj)
}

// --- the reference certificates ------------------------------------------------------------

#[test]
fn reads_a_natural_person_a3_certificate() {
    let info = info("icp-pf-a3");
    assert_eq!(info.icp_brasil, Some(person(Some(CPF))));
    assert_eq!(info.policies, ["2.16.76.1.2.3.1"]);
    assert_eq!(
        info.subject.common_name.as_deref(),
        Some("ANA BEATRIZ SOUZA:12345678901")
    );
}

#[test]
fn reads_a_company_a1_certificate() {
    let info = info("icp-pj-a1");
    assert_eq!(
        info.icp_brasil,
        Some(company(Some(CNPJ), Some(RESPONSIBLE_CPF)))
    );
    assert_eq!(info.policies, ["2.16.76.1.2.1.1"]);
}

#[test]
fn certificates_outside_icp_brasil_have_no_icp_data() {
    for name in [
        "rsa2048",
        "p256",
        "ca",
        "qc-esign-sscd",
        "non-icp-upn",
        "v1",
        "dn-utf8",
    ] {
        assert_eq!(info(name).icp_brasil, None, "{name}");
    }
}

// --- level ------------------------------------------------------------------------------------

#[test]
fn maps_policy_arcs_to_levels() {
    let table: [(u32, IcpLevel); 18] = [
        (0, Other(0)),
        (1, A1),
        (2, A2),
        (3, A3),
        (4, A4),
        (5, Other(5)),
        (100, Other(100)),
        (101, S1),
        (102, S2),
        (103, S3),
        (104, S4),
        (105, Other(105)),
        (302, Other(302)),
        (303, T3),
        (304, T4),
        (305, Other(305)),
        (999, Other(999)),
        (u32::MAX, Other(u32::MAX)),
    ];
    for (arc, level) in table {
        let icp = icp(&format!("icp-level-{arc}"));
        assert_eq!(icp.level, Some(level), "policy 2.16.76.1.2.{arc}.1");
        assert_eq!(
            icp.holder_name,
            Some(format!("LEVEL {arc}")),
            "holder for {arc}"
        );
    }
}

#[test]
fn uses_the_first_icp_policy_and_skips_foreign_ones_before_it() {
    let info = info("icp-multi-policy");
    assert_eq!(
        info.policies,
        ["1.2.3.4", "2.16.76.1.2.3.4", "2.16.76.1.2.1.2"]
    );
    assert_eq!(info.icp_brasil.expect("icp").level, Some(A3));
}

#[test]
fn the_first_icp_policy_wins_even_when_its_arc_is_unknown() {
    let info = info("icp-multi-policy-other-first");
    assert_eq!(info.policies, ["2.16.76.1.2.999.1", "2.16.76.1.2.3.1"]);
    assert_eq!(info.icp_brasil.expect("icp").level, Some(Other(999)));
}

#[test]
fn reads_the_level_from_a_policy_without_further_arcs() {
    // SPEC: the policy is "2.16.76.1.2.<n>.<...>"; a bare 2.16.76.1.2.3 still starts
    // with the ICP prefix, so it is read as n = 3.
    assert_eq!(icp("icp-level-no-subarc").level, Some(A3));
}

#[test]
fn survives_a_policy_arc_beyond_u32() {
    // SPEC is silent (Other(n) holds a u32); the certificate may be accepted or
    // rejected, but nothing may panic.
    let _ = CertInfo::from_der(&cert("icp-level-huge"));
}

#[test]
fn level_is_none_without_an_icp_policy() {
    assert_eq!(icp("icp-san-only").level, None);
    assert_eq!(icp("icp-san-other-arc").level, None);
}

// --- what makes a certificate ICP-Brasil -------------------------------------------------------------

#[test]
fn recognizes_the_policy_alone() {
    assert_eq!(
        info("icp-policy-only").icp_brasil,
        Some(expected(Some(A1), Some("POLICY ONLY"), None, None))
    );
}

#[test]
fn recognizes_a_person_other_name_alone() {
    assert_eq!(
        info("icp-san-only").icp_brasil,
        Some(expected(None, Some("SAN ONLY"), Some(CPF), None))
    );
}

#[test]
fn any_other_name_under_2_16_76_1_3_marks_the_certificate() {
    // 2.16.76.1.3.2 (responsible's name) carries no CPF or CNPJ, but is ICP-Brasil.
    assert_eq!(
        info("icp-san-other-arc").icp_brasil,
        Some(expected(None, Some("OTHER ARC"), None, None))
    );
}

#[test]
fn oids_that_only_share_a_digit_prefix_do_not_count() {
    // Policies 2.16.76.1.20.3 and 2.16.760.1.2.3.1, otherName 2.16.76.1.30.1.
    let info = info("icp-lookalike");
    assert_eq!(info.policies, ["2.16.76.1.20.3", "2.16.760.1.2.3.1"]);
    assert_eq!(info.icp_brasil, None);
}

#[test]
fn other_general_names_and_foreign_other_names_do_not_count() {
    // e-mail, DNS and a Microsoft UPN otherName.
    assert_eq!(info("non-icp-upn").icp_brasil, None);
}

// --- holder name ------------------------------------------------------------------------------------------

#[test]
fn strips_a_trailing_colon_and_digits_from_the_common_name() {
    let cases = [
        ("icp-cn-plain", "MARIA SILVA"),
        ("icp-cn-suffix", "MARIA SILVA"),
        ("icp-cn-nested-colon", "R2:D2"),
        ("icp-cn-accented", "JOSÉ D'ÁVILA"),
        ("icp-pj-a1", "EMPRESA TESTE LTDA"),
    ];
    for (name, holder) in cases {
        assert_eq!(icp(name).holder_name.as_deref(), Some(holder), "{name}");
    }
}

#[test]
fn keeps_a_suffix_that_is_not_only_digits() {
    let cases = [
        ("icp-cn-alpha-suffix", "MARIA SILVA:ABC"),
        ("icp-cn-mixed-suffix", "MARIA SILVA:123ABC"),
        ("icp-cn-space-suffix", "MARIA SILVA: 123"),
    ];
    for (name, holder) in cases {
        assert_eq!(icp(name).holder_name.as_deref(), Some(holder), "{name}");
    }
}

#[test]
fn holder_is_none_without_a_common_name() {
    let icp = icp("icp-no-cn");
    assert_eq!(icp.holder_name, None);
    assert_eq!(icp.level, Some(A1), "still ICP-Brasil through its policy");
}

// --- CPF ------------------------------------------------------------------------------------------------------

#[test]
fn reads_the_cpf_in_every_accepted_string_type() {
    for name in ["icp-pf-a3", "icp-pf-printable", "icp-pf-utf8", "icp-pf-ia5"] {
        assert_eq!(icp(name), person(Some(CPF)), "{name}");
    }
}

#[test]
fn ignores_a_cpf_other_name_of_an_unsupported_string_type() {
    // SPEC: only OCTET STRING, PrintableString, UTF8String and IA5String are read;
    // this one is a BMPString, so it counts as absent.
    assert_eq!(icp("icp-pf-bmp"), person(None));
}

#[test]
fn takes_the_cpf_from_characters_8_to_19() {
    // 19 characters is the shortest usable value; 18 is one short.
    assert_eq!(icp("icp-pf-len-19").cpf.as_deref(), Some(CPF));
    assert_eq!(icp("icp-pf-len-18").cpf, None);
    assert_eq!(icp("icp-pf-len-8").cpf, None, "birth date only");
}

#[test]
fn only_the_eleven_cpf_characters_have_to_be_digits() {
    // "ABCDEFGH" for the birth date and letters after character 19 are fine.
    assert_eq!(icp("icp-pf-odd-birth-tail").cpf.as_deref(), Some(CPF));
    assert_eq!(icp("icp-pf-cpf-letter").cpf, None);
}

#[test]
fn rejects_an_all_zero_cpf() {
    assert_eq!(icp("icp-pf-cpf-zeros").cpf, None);
}

#[test]
fn takes_the_cpf_of_the_responsible_person_when_there_is_no_person_other_name() {
    assert_eq!(icp("icp-only-34").cpf.as_deref(), Some(OTHER_CPF));
    assert_eq!(icp("icp-pj-a1").cpf.as_deref(), Some(RESPONSIBLE_CPF));
}

#[test]
fn the_person_other_name_wins_over_the_responsible_one_in_any_order() {
    // In the certificate 2.16.76.1.3.4 comes first, then 2.16.76.1.3.1.
    assert_eq!(icp("icp-pf-priority").cpf.as_deref(), Some(CPF));
}

#[test]
fn a_present_but_invalid_person_value_is_not_replaced_by_the_responsible_one() {
    // SPEC: "ou, se ausente, do 2.16.76.1.3.4" is read as: fall back only when the
    // person otherName is absent, not when it is present and unusable.
    assert_eq!(icp("icp-pf-invalid-primary").cpf, None);
}

// --- CNPJ ------------------------------------------------------------------------------------------------------

#[test]
fn reads_the_cnpj_in_every_accepted_string_type() {
    for name in ["icp-pj-a1", "icp-pj-printable", "icp-pj-utf8", "icp-pj-ia5"] {
        assert_eq!(
            icp(name),
            company(Some(CNPJ), Some(RESPONSIBLE_CPF)),
            "{name}"
        );
    }
}

#[test]
fn rejects_a_cnpj_that_is_not_exactly_14_digits() {
    for name in ["icp-pj-cnpj-13", "icp-pj-cnpj-15", "icp-pj-cnpj-letter"] {
        assert_eq!(icp(name), company(None, Some(RESPONSIBLE_CPF)), "{name}");
    }
}

#[test]
fn rejects_an_all_zero_cnpj() {
    assert_eq!(
        icp("icp-pj-cnpj-zeros"),
        company(None, Some(RESPONSIBLE_CPF))
    );
}

#[test]
fn a_company_without_a_responsible_person_has_no_cpf() {
    assert_eq!(icp("icp-pj-no-responsible"), company(Some(CNPJ), None));
}

#[test]
fn a_person_certificate_has_no_cnpj() {
    assert_eq!(icp("icp-pf-a3").cnpj, None);
}

// --- masked_cpf and formatted_cnpj ----------------------------------------------------------------------------

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
fn formatting_odd_public_field_values_never_panics() {
    // The fields are public, so a caller can put anything in them.
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
        let _ = with_cpf(text).masked_cpf();
        let _ = with_cnpj(text).formatted_cnpj();
    }
}

// --- IcpLevel and IcpBrasil ------------------------------------------------------------------------------------

#[test]
fn displays_known_levels_by_their_short_name() {
    let names = [
        (A1, "A1"),
        (A2, "A2"),
        (A3, "A3"),
        (A4, "A4"),
        (S1, "S1"),
        (S2, "S2"),
        (S3, "S3"),
        (T3, "T3"),
        (S4, "S4"),
        (T4, "T4"),
    ];
    for (level, name) in names {
        assert_eq!(level.to_string(), name);
    }
}

#[test]
fn displays_unknown_levels_with_their_arc() {
    assert_eq!(Other(999).to_string(), "ICP-Brasil (999)");
    assert_eq!(Other(0).to_string(), "ICP-Brasil (0)");
    assert_eq!(Other(5).to_string(), "ICP-Brasil (5)");
    assert_eq!(Other(u32::MAX).to_string(), "ICP-Brasil (4294967295)");
}

#[test]
fn level_is_copy_eq_and_hashable() {
    let level = A3;
    let copy = level;
    assert_eq!(level, copy);
    assert_ne!(A3, Other(3), "a hand-built Other(3) is not A3");
    assert_ne!(Other(1), Other(2));
    let all = [A1, A2, A3, A4, S1, S2, S3, S4, T3, T4, Other(7)];
    assert_eq!(all.into_iter().collect::<HashSet<_>>().len(), 11);
}

#[test]
fn icp_brasil_defaults_to_nothing_known() {
    assert_eq!(IcpBrasil::default(), expected(None, None, None, None));
}
