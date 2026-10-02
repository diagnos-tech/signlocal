//! Turning certificate bytes into a [`CertInfo`].

use super::extensions::Extensions;
use super::key::PublicKey;
use super::x509::Certificate;
use super::{CertError, CertInfo, DistinguishedName, IcpBrasil, malformed, time};
use crate::fingerprint::Fingerprint;
use crate::hex;

/// Summarises `der` and returns its public key alongside, so the verifier
/// reads the key material without decoding the certificate twice.
pub(crate) fn decode(der: &[u8]) -> Result<(CertInfo, PublicKey<'_>), CertError> {
    let certificate = Certificate::locate(der)?;
    let subject =
        DistinguishedName::from_name(certificate.subject).map_err(|e| malformed("subject", e))?;
    let issuer =
        DistinguishedName::from_name(certificate.issuer).map_err(|e| malformed("issuer", e))?;
    let not_before =
        time::unix_seconds(certificate.not_before).map_err(|e| malformed("notBefore", e))?;
    let not_after =
        time::unix_seconds(certificate.not_after).map_err(|e| malformed("notAfter", e))?;
    let public_key = PublicKey::from_spki(&certificate.public_key)?;
    let extensions = Extensions::parse(&certificate.extensions)?;
    let icp_brasil = IcpBrasil::detect(
        &extensions.policies,
        &extensions.other_names,
        subject.common_name.as_deref(),
    );

    let info = CertInfo {
        fingerprint: Fingerprint::of(der),
        subject,
        issuer,
        serial_hex: serial_hex(certificate.serial),
        not_before,
        not_after,
        key: public_key.kind(),
        key_usage: extensions.key_usage,
        extended_key_usage: extensions.extended_key_usage,
        policies: extensions.policies,
        is_ca: extensions.is_ca,
        icp_brasil,
        qualified: extensions.qualified,
    };
    Ok((info, public_key))
}

/// Hex of the serial's value: its content octets without leading `00`s,
/// keeping at least one octet. For a DER-minimal positive serial that only
/// drops the sign byte (`00 80` reads `80`), matching what OS viewers and
/// OpenSSL print; a serial padded with redundant zeros reads like the
/// minimal one. A negative serial (non-conforming, but issued) keeps its
/// two's-complement octets, as Windows and macOS show it.
fn serial_hex(serial: &[u8]) -> String {
    let start = serial
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(serial.len())
        .min(serial.len().saturating_sub(1));
    hex::lower(serial.get(start..).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_drops_leading_zero_octets_only() {
        assert_eq!(serial_hex(&[0x00, 0x80]), "80");
        assert_eq!(serial_hex(&[0x12, 0x34]), "1234");
        assert_eq!(serial_hex(&[0x01, 0x02]), "0102");
        assert_eq!(serial_hex(&[0x00]), "00");
        assert_eq!(serial_hex(&[0x00, 0x00]), "00");
        assert_eq!(serial_hex(&[0x7f]), "7f");
        assert_eq!(serial_hex(&[0x00, 0x00, 0x05]), "05");
        assert_eq!(serial_hex(&[0xff, 0x7f]), "ff7f");
    }
}
