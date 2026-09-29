//! Turning a decoded X.509 certificate into a [`CertInfo`].

use der::Decode;
use x509_cert::Certificate;
use x509_cert::time::Time;

use super::asn1::malformed;
use super::extensions::Extensions;
use super::{CertError, CertInfo, DistinguishedName, IcpBrasil, PublicKeyKind};
use crate::fingerprint::Fingerprint;
use crate::hex;

/// Decodes `der` as a certificate, rejecting trailing bytes.
///
/// SPEC: decoding is as strict as the `x509-cert` crate, so a certificate
/// with a date before 1970 or a serial number longer than 21 octets is
/// `Malformed`; both are far outside what ICP-Brasil and eIDAS CAs issue.
///
/// Kept apart from [`CertInfo::from_certificate`] so the verifier can reuse
/// the decoded key material without decoding the certificate twice.
pub(crate) fn decode_certificate(der: &[u8]) -> Result<Certificate, CertError> {
    Certificate::from_der(der).map_err(|e| malformed("certificate", e))
}

impl CertInfo {
    /// Summarises an already decoded certificate; `der` must be the bytes it
    /// was decoded from, since the fingerprint is taken over them verbatim.
    pub(crate) fn from_certificate(der: &[u8], cert: &Certificate) -> Result<CertInfo, CertError> {
        let tbs = cert.tbs_certificate();
        let subject = DistinguishedName::from_name(tbs.subject());
        let extensions = Extensions::parse(tbs.extensions().map(Vec::as_slice))?;
        let icp_brasil = IcpBrasil::detect(
            &extensions.policies,
            &extensions.other_names,
            subject.common_name.as_deref(),
        );

        Ok(CertInfo {
            fingerprint: Fingerprint::of(der),
            issuer: DistinguishedName::from_name(tbs.issuer()),
            serial_hex: serial_hex(tbs.serial_number().as_bytes()),
            not_before: unix_seconds(tbs.validity().not_before)?,
            not_after: unix_seconds(tbs.validity().not_after)?,
            key: PublicKeyKind::from_spki(tbs.subject_public_key_info())?,
            key_usage: extensions.key_usage,
            extended_key_usage: extensions.extended_key_usage,
            policies: extensions.policies,
            is_ca: extensions.is_ca,
            icp_brasil,
            qualified: extensions.qualified,
            subject,
        })
    }
}

/// Hex of the serial's content octets, minus the `00` that DER adds in front
/// of a value whose top bit is set (so `00 80` reads `80`, like OS viewers).
fn serial_hex(serial: &[u8]) -> String {
    match serial.split_first() {
        Some((0, rest)) if !rest.is_empty() => hex::lower(rest),
        _ => hex::lower(serial),
    }
}

fn unix_seconds(time: Time) -> Result<i64, CertError> {
    i64::try_from(time.to_unix_duration().as_secs()).map_err(|e| malformed("validity", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_drops_only_the_sign_byte() {
        assert_eq!(serial_hex(&[0x00, 0x80]), "80");
        assert_eq!(serial_hex(&[0x12, 0x34]), "1234");
        assert_eq!(serial_hex(&[0x00]), "00");
        assert_eq!(serial_hex(&[0x7f]), "7f");
        assert_eq!(serial_hex(&[0x00, 0xff, 0x00]), "ff00");
        assert_eq!(serial_hex(&[]), "");
    }

    #[test]
    fn garbage_is_malformed_not_a_panic() {
        for bytes in [
            &[][..],
            &[0x30][..],
            &[0x30, 0x00][..],
            b"not a certificate",
        ] {
            assert!(matches!(
                decode_certificate(bytes),
                Err(CertError::Malformed(_))
            ));
        }
    }
}
