//! `Debug` for the types that carry personal identifiers.
//!
//! A `{:?}` of a certificate summary ends up in panic messages, test output
//! and, by accident, in logs. Nothing may log a summary on purpose (names are
//! personal data too), but as a safety net the document numbers print masked:
//! the CPF, the subject `serialNumber` (a national ID number) and the
//! `:<digits>` ICP-Brasil appends to the CN (a CPF or CNPJ).

use std::fmt;

use super::{DistinguishedName, IcpBrasil};

/// Printed in place of an identifier that has no safe masked form.
const REDACTED: &str = "<redacted>";

impl fmt::Debug for IcpBrasil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cpf = self
            .cpf
            .as_ref()
            .map(|_| self.masked_cpf().unwrap_or_else(|| REDACTED.to_owned()));
        f.debug_struct("IcpBrasil")
            .field("level", &self.level)
            .field("holder_name", &self.holder_name)
            .field("cpf", &cpf)
            .field("cnpj", &self.cnpj)
            .finish()
    }
}

impl fmt::Debug for DistinguishedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DistinguishedName")
            .field("common_name", &self.common_name.as_deref().map(masked_cn))
            .field("organization", &self.organization)
            .field("organizational_units", &self.organizational_units)
            .field("country", &self.country)
            .field("given_name", &self.given_name)
            .field("surname", &self.surname)
            .field(
                "serial_number",
                &self.serial_number.as_ref().map(|_| REDACTED),
            )
            .finish()
    }
}

/// `"ANA:12345678901"` → `"ANA:<redacted>"`; other names unchanged.
fn masked_cn(cn: &str) -> String {
    match cn.rsplit_once(':') {
        Some((name, id)) if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{name}:{REDACTED}")
        }
        _ => cn.to_owned(),
    }
}
