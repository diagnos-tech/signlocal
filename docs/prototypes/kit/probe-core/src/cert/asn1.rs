//! ASN.1 helpers shared by the certificate parsers.

use std::fmt::Display;

use der::Decode;
use der::Reader;
use der::asn1::{AnyRef, ObjectIdentifier};

use super::CertError;

/// Wraps a decoding failure with the name of the structure that failed, so a
/// `Malformed` message says which part of the certificate is broken.
pub(super) fn malformed(what: &str, err: impl Display) -> CertError {
    CertError::Malformed(format!("{what}: {err}"))
}

/// Splits `SEQUENCE { OBJECT IDENTIFIER, ANY OPTIONAL }`.
///
/// This is the shape of both `PolicyInformation` (the ANY being the
/// qualifiers) and `QCStatement` (the ANY being the statement info). Anything
/// after the optional element makes the sequence malformed.
pub(super) fn oid_with_optional_info(
    element: AnyRef<'_>,
) -> der::Result<(ObjectIdentifier, Option<AnyRef<'_>>)> {
    element.sequence(|reader| {
        let oid = ObjectIdentifier::decode(reader)?;
        let info = if reader.is_finished() {
            None
        } else {
            Some(AnyRef::decode(reader)?)
        };
        Ok((oid, info))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_oid_and_optional_info() {
        // SEQUENCE { OID 2.5.29.32, NULL }
        let with_info = [0x30, 0x07, 0x06, 0x03, 0x55, 0x1d, 0x20, 0x05, 0x00];
        let (oid, info) = oid_with_optional_info(AnyRef::from_der(&with_info).unwrap()).unwrap();
        assert_eq!(oid.to_string(), "2.5.29.32");
        assert!(info.is_some());

        let without = [0x30, 0x05, 0x06, 0x03, 0x55, 0x1d, 0x20];
        let (oid, info) = oid_with_optional_info(AnyRef::from_der(&without).unwrap()).unwrap();
        assert_eq!(oid.to_string(), "2.5.29.32");
        assert_eq!(info, None);
    }

    #[test]
    fn rejects_extra_elements_and_wrong_shapes() {
        let three = [
            0x30, 0x09, 0x06, 0x03, 0x55, 0x1d, 0x20, 0x05, 0x00, 0x05, 0x00,
        ];
        assert!(oid_with_optional_info(AnyRef::from_der(&three).unwrap()).is_err());

        let not_oid_first = [0x30, 0x03, 0x02, 0x01, 0x05];
        assert!(oid_with_optional_info(AnyRef::from_der(&not_oid_first).unwrap()).is_err());

        let not_a_sequence = [0x31, 0x05, 0x06, 0x03, 0x55, 0x1d, 0x20];
        assert!(oid_with_optional_info(AnyRef::from_der(&not_a_sequence).unwrap()).is_err());
    }
}
