//! SubjectAltName `otherName` entries, where ICP-Brasil keeps CPF and CNPJ.

use super::der::{
    self, DerError, IA5_STRING, OBJECT_IDENTIFIER, OCTET_STRING, PRINTABLE_STRING, Reader,
    SEQUENCE, Tlv, UTF8_STRING, explicit,
};
use super::oid;

/// One `otherName` of a SubjectAltName.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OtherName {
    /// Dotted type-id.
    pub oid: String,
    /// The value as ASCII text; `None` if it has an unsupported type or is
    /// not ASCII.
    pub value: Option<String>,
}

/// The `otherName` choice of `GeneralName`, and also the EXPLICIT wrapper
/// around an `otherName`'s value.
const OTHER_NAME: u8 = explicit(0);

/// Extracts the `otherName` entries of a SubjectAltName extension value.
///
/// Other `GeneralName` choices are skipped without being inspected, so a name
/// form this crate does not model cannot make the certificate unreadable. So
/// is an `otherName` whose type-id cannot even be printed: it cannot be one
/// of the ICP-Brasil ones.
pub(super) fn other_names(extension_value: &[u8]) -> Result<Vec<OtherName>, DerError> {
    let mut names = Vec::new();
    for name in Reader::new(der::single(extension_value, SEQUENCE)?) {
        let name = name?;
        if name.tag != OTHER_NAME {
            continue;
        }
        let (type_id, value) = decode_other_name(name.content)?;
        if let Some(oid) = oid::to_dotted(type_id) {
            names.push(OtherName {
                oid,
                value: ascii_text(value),
            });
        }
    }
    Ok(names)
}

/// `OtherName ::= SEQUENCE { type-id OID, value [0] EXPLICIT ANY }`, with the
/// outer SEQUENCE header already replaced by the `[0]` of `GeneralName`.
fn decode_other_name(content: &[u8]) -> Result<(&[u8], Tlv<'_>), DerError> {
    let mut fields = Reader::new(content);
    let type_id = fields.read(OBJECT_IDENTIFIER)?;
    let mut wrapper = Reader::new(fields.read(OTHER_NAME)?);
    fields.finish()?;
    let value = wrapper.read_any()?;
    wrapper.finish()?;
    Ok((type_id, value))
}

/// ICP-Brasil values are digits and letters. The four string-like types seen
/// in practice are accepted; anything else, or non-ASCII content, counts as
/// no value rather than being decoded lossily, so byte offsets into the text
/// are also character offsets.
fn ascii_text(value: Tlv<'_>) -> Option<String> {
    match value.tag {
        OCTET_STRING | PRINTABLE_STRING | UTF8_STRING | IA5_STRING => value
            .content
            .is_ascii()
            .then(|| value.content.iter().map(|&b| char::from(b)).collect()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TLV with a short length.
    fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut out = vec![tag, u8::try_from(content.len()).unwrap()];
        out.extend_from_slice(content);
        out
    }

    /// `[0] { OID 2.16.76.1.3.1, [0] { <value> } }` as one GeneralName.
    fn other_name(value: &[u8]) -> Vec<u8> {
        let oid = tlv(0x06, &[0x60, 0x4c, 0x01, 0x03, 0x01]);
        let wrapped = tlv(0xa0, value);
        tlv(0xa0, &[oid, wrapped].concat())
    }

    fn general_names(names: &[Vec<u8>]) -> Vec<u8> {
        tlv(0x30, &names.concat())
    }

    #[test]
    fn reads_the_supported_string_types_as_text() {
        for tag in [0x04, 0x13, 0x0c, 0x16] {
            let san = general_names(&[other_name(&tlv(tag, b"0123"))]);
            let names = other_names(&san).unwrap();
            assert_eq!(
                names,
                vec![OtherName {
                    oid: "2.16.76.1.3.1".into(),
                    value: Some("0123".into())
                }],
                "tag {tag:#x}"
            );
        }
    }

    #[test]
    fn unsupported_value_types_and_non_ascii_give_no_value() {
        for value in [
            tlv(0x02, &[5]),
            tlv(0x0c, "ã".as_bytes()),
            tlv(0x1e, &[0, 0x41]),
        ] {
            let names = other_names(&general_names(&[other_name(&value)])).unwrap();
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].value, None);
        }
    }

    #[test]
    fn other_general_names_and_unprintable_type_ids_are_skipped() {
        let email = tlv(0x81, b"a@b.c");
        let dns = tlv(0x82, b"example.com");
        let x400 = tlv(0xa3, &[0x30, 0x00]);
        let bad_id = tlv(
            0xa0,
            &[tlv(0x06, &[0x80, 0x01]), tlv(0xa0, &tlv(0x0c, b"1"))].concat(),
        );
        let san = general_names(&[email, x400, other_name(&tlv(0x0c, b"1")), bad_id, dns]);
        assert_eq!(other_names(&san).unwrap().len(), 1);
    }

    #[test]
    fn malformed_other_names_are_errors() {
        let oid = tlv(0x06, &[0x60, 0x4c, 0x01, 0x03, 0x01]);
        let no_value = tlv(0xa0, &oid);
        let wrong_wrapper = tlv(0xa0, &[oid.clone(), tlv(0xa1, &tlv(0x0c, b"1"))].concat());
        let two_values = tlv(
            0xa0,
            &[oid, tlv(0xa0, &[tlv(0x0c, b"1"), tlv(0x0c, b"2")].concat())].concat(),
        );
        for bad in [no_value, wrong_wrapper, two_values] {
            assert!(other_names(&general_names(&[bad])).is_err());
        }
        assert!(other_names(&tlv(0x04, b"x")).is_err());
        assert!(other_names(&[]).is_err());
    }
}
