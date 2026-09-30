//! ICP-Brasil markers: certificate level and holder identifiers (DOC-ICP-04).

mod level;

pub use level::IcpLevel;

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
///
/// `Debug` prints the CPF masked, so a stray `{:?}` cannot leak it.
#[derive(Clone, Default, PartialEq, Eq)]
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
            level: IcpLevel::from_policies(policies, POLICY_PREFIX),
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

/// Drops the `:<digits>` that ICP-Brasil appends to the CN (the holder's CPF
/// or the company's CNPJ) so the name reads naturally. Only the last `:`
/// counts and at least one digit must follow it, so `"ANA:"` and
/// `"ANA:12x"` are left untouched and `"A:1:2"` becomes `"A:1"`.
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
/// A person value that is present but unusable (too short, all zeros, not
/// digits, unreadable type) yields `None` rather than the responsible
/// person's CPF: DOC-ICP-04 zero-fills data that is not available, and
/// showing someone else's CPF as the holder's would misattribute it.
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

#[cfg(test)]
mod tests {
    use super::without_id_suffix;

    #[test]
    fn only_a_last_colon_followed_by_digits_is_dropped() {
        let cases = [
            ("ANA BEATRIZ SOUZA:12345678901", "ANA BEATRIZ SOUZA"),
            ("A:B:99", "A:B"),
            ("A:1:2", "A:1"),
            ("ANA SOUZA", "ANA SOUZA"),
            ("ANA:12x", "ANA:12x"),
            ("ANA:", "ANA:"),
            ("ANA:１２", "ANA:１２"),
            (":123", ""),
        ];
        for (common_name, holder) in cases {
            assert_eq!(without_id_suffix(common_name), holder, "{common_name}");
        }
    }
}
