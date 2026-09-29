//! SubjectAltName `otherName` entries, where ICP-Brasil keeps CPF and CNPJ.

use der::asn1::{AnyRef, ObjectIdentifier};
use der::{Decode, Tag, TagNumber, Tagged};

/// One `otherName` of a SubjectAltName.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OtherName {
    /// Dotted type-id.
    pub oid: String,
    /// The value as ASCII text; `None` if it has an unsupported type or is
    /// not ASCII.
    pub value: Option<String>,
}

/// `[0]` constructed: the `otherName` choice of `GeneralName`, and also the
/// EXPLICIT wrapper around an `otherName`'s value.
const CONTEXT_0: Tag = Tag::ContextSpecific {
    constructed: true,
    number: TagNumber(0),
};

/// Extracts the `otherName` entries of a SubjectAltName extension value.
///
/// Other `GeneralName` choices are skipped without being inspected, so a name
/// form this crate does not model cannot make the certificate unreadable.
pub(super) fn other_names(extension_value: &[u8]) -> der::Result<Vec<OtherName>> {
    Vec::<AnyRef>::from_der(extension_value)?
        .into_iter()
        .filter(|name| name.tag() == CONTEXT_0)
        .map(|name| decode_other_name(name.value()))
        .collect()
}

/// `OtherName ::= SEQUENCE { type-id OID, value [0] EXPLICIT ANY }`, with the
/// outer SEQUENCE header already replaced by the `[0]` of `GeneralName`.
fn decode_other_name(content: &[u8]) -> der::Result<OtherName> {
    let mut reader = der::SliceReader::new(content)?;
    let oid: ObjectIdentifier = ObjectIdentifier::decode(&mut reader)?;
    let wrapper = AnyRef::decode(&mut reader)?;
    der::Reader::finish(reader)?;
    wrapper.tag().assert_eq(CONTEXT_0)?;

    let value = AnyRef::from_der(wrapper.value())?;
    Ok(OtherName {
        oid: oid.to_string(),
        value: ascii_text(value),
    })
}

/// ICP-Brasil values are digits and letters. The four string-like types seen
/// in practice are accepted; anything else, or non-ASCII content, is dropped
/// so byte offsets into the text are also character offsets.
///
/// SPEC: "read as ASCII text" means a value with any non-ASCII byte has no
/// text (as if its type were unsupported) rather than being decoded lossily.
fn ascii_text(value: AnyRef<'_>) -> Option<String> {
    match value.tag() {
        Tag::OctetString | Tag::PrintableString | Tag::Utf8String | Tag::Ia5String => {
            let bytes = value.value();
            bytes
                .is_ascii()
                .then(|| bytes.iter().map(|&b| char::from(b)).collect())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TLV with a short length.
    fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut out = vec![tag, content.len() as u8];
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
    fn other_general_names_are_skipped() {
        let email = tlv(0x81, b"a@b.c");
        let dns = tlv(0x82, b"example.com");
        let x400 = tlv(0xa3, &[0x30, 0x00]);
        let san = general_names(&[email, x400, other_name(&tlv(0x0c, b"1")), dns]);
        assert_eq!(other_names(&san).unwrap().len(), 1);
    }

    #[test]
    fn malformed_other_names_are_errors() {
        // Missing the [0] value wrapper.
        let no_value = tlv(0xa0, &tlv(0x06, &[0x60, 0x4c, 0x01, 0x03, 0x01]));
        assert!(other_names(&general_names(&[no_value])).is_err());
        // Wrapper with the wrong tag.
        let bad_wrapper = tlv(
            0xa0,
            &[
                tlv(0x06, &[0x60, 0x4c, 0x01, 0x03, 0x01]),
                tlv(0xa1, &tlv(0x0c, b"1")),
            ]
            .concat(),
        );
        assert!(other_names(&general_names(&[bad_wrapper])).is_err());
        // Not a SEQUENCE OF at all.
        assert!(other_names(&tlv(0x04, b"x")).is_err());
        assert!(other_names(&[]).is_err());
    }
}
