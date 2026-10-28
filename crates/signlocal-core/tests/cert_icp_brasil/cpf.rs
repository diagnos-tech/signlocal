//! CPF.

use super::*;

#[test]
fn reads_the_cpf_in_every_accepted_string_type() {
    for name in ["icp-pf-a3", "icp-pf-printable", "icp-pf-utf8", "icp-pf-ia5"] {
        assert_eq!(icp(name), person(Some(CPF)), "{name}");
    }
}

#[test]
fn ignores_a_cpf_other_name_of_an_unsupported_string_type() {
    // Only OCTET STRING, PrintableString, UTF8String and IA5String are read;
    // this one is a BMPString, so the CPF is unknown.
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
    // Fall back only when the person otherName is absent, not when it is
    // present and unusable: the responsible person is someone else.
    assert_eq!(icp("icp-pf-invalid-primary").cpf, None);
}
