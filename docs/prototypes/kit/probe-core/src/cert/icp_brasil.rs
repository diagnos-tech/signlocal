//! ICP-Brasil markers: certificate level and holder identifiers (DOC-ICP-04).

use std::fmt;

use super::san::OtherName;

const POLICY_PREFIX: &str = "2.16.76.1.2.";
const OTHER_NAME_PREFIX: &str = "2.16.76.1.3.";
const OID_PERSON_DATA: &str = "2.16.76.1.3.1";
const OID_COMPANY_ID: &str = "2.16.76.1.3.3";
const OID_RESPONSIBLE_DATA: &str = "2.16.76.1.3.4";

/// A birth date (`ddmmyyyy`) precedes the CPF inside the person-data values.
const BIRTH_DATE_LEN: usize = 8;
const CPF_LEN: usize = 11;
const CNPJ_LEN: usize = 14;

/// ICP-Brasil data extracted from policies and SubjectAltName `otherName`s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IcpBrasil {
    /// From the first `2.16.76.1.2.<n>` policy.
    pub level: Option<IcpLevel>,
    /// Subject CN without the trailing `:<digits>` ICP-Brasil appends.
    pub holder_name: Option<String>,
    /// 11 digits: the holder's CPF, or the CPF of a company's responsible person.
    pub cpf: Option<String>,
    /// 14 digits.
    pub cnpj: Option<String>,
}

impl IcpBrasil {
    /// CPF masked like gov.br does: `***.456.789-**`.
    ///
    /// Enough for the holder to recognize their own certificate, useless to
    /// anyone reading over their shoulder.
    pub fn masked_cpf(&self) -> Option<String> {
        let cpf = digits(self.cpf.as_deref()?, CPF_LEN)?;
        Some(format!("***.{}.{}-**", cpf.get(3..6)?, cpf.get(6..9)?))
    }

    /// CNPJ in its usual `12.345.678/0001-95` form. A company identifier is
    /// public data, so it is not masked.
    pub fn formatted_cnpj(&self) -> Option<String> {
        let cnpj = digits(self.cnpj.as_deref()?, CNPJ_LEN)?;
        Some(format!(
            "{}.{}.{}/{}-{}",
            cnpj.get(0..2)?,
            cnpj.get(2..5)?,
            cnpj.get(5..8)?,
            cnpj.get(8..12)?,
            cnpj.get(12..14)?,
        ))
    }

    /// Builds the summary if the certificate is ICP-Brasil, which is the case
    /// when a policy sits under `2.16.76.1.2.` or an `otherName` under
    /// `2.16.76.1.3.`. Either marker alone is enough: some issuers omit one.
    pub(super) fn detect(
        policies: &[String],
        other_names: &[OtherName],
        subject_common_name: Option<&str>,
    ) -> Option<Self> {
        let by_policy = policies.iter().any(|p| p.starts_with(POLICY_PREFIX));
        let by_name = other_names
            .iter()
            .any(|n| n.oid.starts_with(OTHER_NAME_PREFIX));
        if !(by_policy || by_name) {
            return None;
        }
        Some(Self {
            level: level(policies),
            holder_name: subject_common_name.map(without_id_suffix),
            cpf: cpf(other_names),
            cnpj: cnpj(other_names),
        })
    }
}

/// `text` if it is exactly `len` ASCII digits.
fn digits(text: &str, len: usize) -> Option<&str> {
    (text.len() == len && text.bytes().all(|b| b.is_ascii_digit())).then_some(text)
}

fn is_all_zero(digits: &str) -> bool {
    digits.bytes().all(|b| b == b'0')
}

fn level(policies: &[String]) -> Option<IcpLevel> {
    policies.iter().find_map(|policy| {
        let arc = policy.strip_prefix(POLICY_PREFIX)?.split('.').next()?;
        arc.parse::<u32>().ok().map(IcpLevel::from_arc)
    })
}

/// Drops the `:<digits>` that ICP-Brasil appends to the CN (the holder's CPF
/// or the company's CNPJ) so the name reads naturally.
///
/// SPEC: only the last `:` counts and at least one digit must follow it, so
/// `"ANA:"` and `"ANA:12x"` are left untouched.
fn without_id_suffix(common_name: &str) -> String {
    match common_name.rsplit_once(':') {
        Some((name, suffix))
            if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) =>
        {
            name.to_owned()
        }
        _ => common_name.to_owned(),
    }
}

fn first_value<'a>(other_names: &'a [OtherName], oid: &str) -> Option<&'a OtherName> {
    other_names.iter().find(|n| n.oid == oid)
}

/// The CPF sits after the birth date in the person-data value. The
/// responsible-person value (used by company certificates) has the same
/// layout and is consulted only when the person's own value is absent.
///
/// SPEC: "absent" is literal. A `2.16.76.1.3.1` entry that is present but
/// unusable (too short, all zeros, not digits) yields `None` and does not
/// fall back to `2.16.76.1.3.4`.
fn cpf(other_names: &[OtherName]) -> Option<String> {
    let entry = first_value(other_names, OID_PERSON_DATA)
        .or_else(|| first_value(other_names, OID_RESPONSIBLE_DATA))?;
    let value = entry.value.as_deref()?;
    let cpf = digits(
        value.get(BIRTH_DATE_LEN..BIRTH_DATE_LEN + CPF_LEN)?,
        CPF_LEN,
    )?;
    (!is_all_zero(cpf)).then(|| cpf.to_owned())
}

fn cnpj(other_names: &[OtherName]) -> Option<String> {
    let value = first_value(other_names, OID_COMPANY_ID)?.value.as_deref()?;
    let cnpj = digits(value, CNPJ_LEN)?;
    (!is_all_zero(cnpj)).then(|| cnpj.to_owned())
}

/// Certificate type from the ICP-Brasil policy OID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IcpLevel {
    A1,
    A2,
    A3,
    A4,
    S1,
    S2,
    S3,
    S4,
    T3,
    T4,
    /// A policy arc this crate does not know yet.
    Other(u32),
}

impl IcpLevel {
    fn from_arc(arc: u32) -> Self {
        match arc {
            1 => Self::A1,
            2 => Self::A2,
            3 => Self::A3,
            4 => Self::A4,
            101 => Self::S1,
            102 => Self::S2,
            103 => Self::S3,
            104 => Self::S4,
            303 => Self::T3,
            304 => Self::T4,
            other => Self::Other(other),
        }
    }
}

impl fmt::Display for IcpLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::A4 => "A4",
            Self::S1 => "S1",
            Self::S2 => "S2",
            Self::S3 => "S3",
            Self::S4 => "S4",
            Self::T3 => "T3",
            Self::T4 => "T4",
            Self::Other(arc) => return write!(f, "ICP-Brasil ({arc})"),
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn other(oid: &str, value: &str) -> OtherName {
        OtherName {
            oid: oid.to_owned(),
            value: Some(value.to_owned()),
        }
    }

    fn policies(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    fn detect(policies: &[&str], names: &[OtherName], cn: Option<&str>) -> Option<IcpBrasil> {
        IcpBrasil::detect(&self::policies(policies), names, cn)
    }

    #[test]
    fn not_icp_without_either_marker() {
        assert_eq!(detect(&["2.16.76.1.2"], &[], None), None);
        assert_eq!(
            detect(&["2.5.29.32.0", "2.16.840.1.101.3.2.1"], &[], Some("X")),
            None
        );
        assert_eq!(detect(&[], &[other("2.16.76.1.4.1", "x")], None), None);
    }

    #[test]
    fn either_marker_is_enough() {
        assert!(detect(&["2.16.76.1.2.3.16"], &[], None).is_some());
        assert!(detect(&[], &[other("2.16.76.1.3.2", "NAME")], None).is_some());
    }

    #[test]
    fn levels_come_from_the_first_icp_policy() {
        let cases = [
            ("2.16.76.1.2.1.16", IcpLevel::A1),
            ("2.16.76.1.2.2", IcpLevel::A2),
            ("2.16.76.1.2.3.5", IcpLevel::A3),
            ("2.16.76.1.2.4.1", IcpLevel::A4),
            ("2.16.76.1.2.101.7", IcpLevel::S1),
            ("2.16.76.1.2.102", IcpLevel::S2),
            ("2.16.76.1.2.103", IcpLevel::S3),
            ("2.16.76.1.2.104", IcpLevel::S4),
            ("2.16.76.1.2.303", IcpLevel::T3),
            ("2.16.76.1.2.304.9", IcpLevel::T4),
            ("2.16.76.1.2.7.1", IcpLevel::Other(7)),
            ("2.16.76.1.2.0", IcpLevel::Other(0)),
        ];
        for (policy, level) in cases {
            let icp = detect(&["2.5.29.32.0", policy, "2.16.76.1.2.1"], &[], None).unwrap();
            assert_eq!(icp.level, Some(level), "{policy}");
        }
    }

    #[test]
    fn level_is_absent_when_only_other_names_mark_icp() {
        let icp = detect(&["2.5.29.32.0"], &[other("2.16.76.1.3.1", "x")], None).unwrap();
        assert_eq!(icp.level, None);
    }

    #[test]
    fn level_display() {
        assert_eq!(IcpLevel::A1.to_string(), "A1");
        assert_eq!(IcpLevel::S4.to_string(), "S4");
        assert_eq!(IcpLevel::T3.to_string(), "T3");
        assert_eq!(IcpLevel::Other(7).to_string(), "ICP-Brasil (7)");
    }

    #[test]
    fn holder_name_loses_the_id_suffix_only() {
        let name = |cn| detect(&["2.16.76.1.2.1"], &[], cn).unwrap().holder_name;
        assert_eq!(
            name(Some("ANA BEATRIZ SOUZA:12345678901")).as_deref(),
            Some("ANA BEATRIZ SOUZA")
        );
        assert_eq!(
            name(Some("ACME LTDA:12345678000195")).as_deref(),
            Some("ACME LTDA")
        );
        assert_eq!(name(Some("ANA SOUZA")).as_deref(), Some("ANA SOUZA"));
        assert_eq!(name(Some("A:B:99")).as_deref(), Some("A:B"));
        assert_eq!(name(Some("ANA:12x")).as_deref(), Some("ANA:12x"));
        assert_eq!(name(Some("ANA:")).as_deref(), Some("ANA:"));
        assert_eq!(name(None), None);
    }

    #[test]
    fn cpf_is_read_after_the_birth_date() {
        let names = [other("2.16.76.1.3.1", "010119901234567890112345678901234")];
        let icp = detect(&[], &names, None).unwrap();
        assert_eq!(icp.cpf.as_deref(), Some("12345678901"));
        assert_eq!(icp.masked_cpf().as_deref(), Some("***.456.789-**"));
    }

    #[test]
    fn responsible_cpf_is_the_fallback_and_person_cpf_wins() {
        let responsible = other("2.16.76.1.3.4", "020219809876543210900000000000");
        let person = other("2.16.76.1.3.1", "010119901234567890100000000000");
        let icp = detect(&[], std::slice::from_ref(&responsible), None).unwrap();
        assert_eq!(icp.cpf.as_deref(), Some("98765432109"));

        let icp = detect(&[], &[responsible, person], None).unwrap();
        assert_eq!(icp.cpf.as_deref(), Some("12345678901"));
    }

    #[test]
    fn invalid_cpf_values_are_dropped() {
        let cpf = |value: &str| {
            detect(&[], &[other("2.16.76.1.3.1", value)], None)
                .unwrap()
                .cpf
        };
        assert_eq!(cpf("0101199012345"), None, "too short");
        assert_eq!(
            cpf("01011990123456789"),
            None,
            "17 characters: one digit missing"
        );
        assert_eq!(cpf("01011990123456789a1"), None, "non digit inside the 11");
        assert_eq!(cpf("01011990123456789 1"), None, "space inside the 11");
        assert_eq!(cpf("0101199000000000000"), None, "all zeros");
        assert_eq!(
            cpf("0101199012345678901").as_deref(),
            Some("12345678901"),
            "exactly 19"
        );
        assert_eq!(
            cpf("abcdefgh12345678901").as_deref(),
            Some("12345678901"),
            "the birth date part is not validated"
        );
    }

    #[test]
    fn cpf_without_a_readable_value_is_none() {
        let names = [OtherName {
            oid: "2.16.76.1.3.1".into(),
            value: None,
        }];
        assert_eq!(detect(&[], &names, None).unwrap().cpf, None);
    }

    #[test]
    fn cnpj_needs_exactly_fourteen_digits() {
        let cnpj = |value: &str| {
            detect(&[], &[other("2.16.76.1.3.3", value)], None)
                .unwrap()
                .cnpj
        };
        assert_eq!(cnpj("12345678000195").as_deref(), Some("12345678000195"));
        assert_eq!(cnpj("1234567800019"), None);
        assert_eq!(cnpj("123456780001955"), None);
        assert_eq!(cnpj("1234567800019x"), None);
        assert_eq!(cnpj("00000000000000"), None);
    }

    #[test]
    fn formatting_helpers() {
        let icp = IcpBrasil {
            cpf: Some("12345678901".into()),
            cnpj: Some("12345678000195".into()),
            ..IcpBrasil::default()
        };
        assert_eq!(icp.masked_cpf().as_deref(), Some("***.456.789-**"));
        assert_eq!(icp.formatted_cnpj().as_deref(), Some("12.345.678/0001-95"));

        let empty = IcpBrasil::default();
        assert_eq!(empty.masked_cpf(), None);
        assert_eq!(empty.formatted_cnpj(), None);
    }

    #[test]
    fn formatting_helpers_refuse_malformed_public_fields() {
        let icp = IcpBrasil {
            cpf: Some("123".into()),
            cnpj: Some("12ã45678000195".into()),
            ..IcpBrasil::default()
        };
        assert_eq!(icp.masked_cpf(), None);
        assert_eq!(icp.formatted_cnpj(), None);
    }
}
