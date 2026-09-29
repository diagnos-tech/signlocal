//! Text extraction from X.501 string types.

use der::Tag;

/// Decodes the value of a distinguished-name attribute into text.
///
/// UTF8String, PrintableString and IA5String are read as UTF-8; BMPString as
/// UTF-16BE; TeletexString as Latin-1 (what CAs that still use it actually
/// store). Any other type, or bytes that are not valid for their type, yield
/// `None` so the attribute is skipped instead of shown garbled.
pub(super) fn decode(tag: Tag, bytes: &[u8]) -> Option<String> {
    match tag {
        Tag::Utf8String | Tag::PrintableString | Tag::Ia5String => {
            std::str::from_utf8(bytes).ok().map(str::to_owned)
        }
        Tag::BmpString => {
            let (pairs, odd_byte) = bytes.as_chunks::<2>();
            // An odd trailing byte means the value is not UTF-16 at all.
            if !odd_byte.is_empty() {
                return None;
            }
            let units = pairs
                .iter()
                .map(|&pair| u16::from_be_bytes(pair))
                .collect::<Vec<u16>>();
            String::from_utf16(&units).ok()
        }
        Tag::TeletexString => Some(bytes.iter().map(|&b| char::from(b)).collect()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_printable_and_ia5_are_read_as_utf8() {
        for tag in [Tag::Utf8String, Tag::PrintableString, Tag::Ia5String] {
            assert_eq!(decode(tag, "João Ç".as_bytes()).as_deref(), Some("João Ç"));
        }
        assert_eq!(decode(Tag::Utf8String, &[0xff, 0xfe]), None);
    }

    #[test]
    fn bmp_is_utf16_big_endian() {
        let bytes = [0x00, 0x4a, 0x00, 0xe3, 0x00, 0x6f];
        assert_eq!(decode(Tag::BmpString, &bytes).as_deref(), Some("Jão"));
        // Surrogate pair for U+1F600.
        assert_eq!(
            decode(Tag::BmpString, &[0xd8, 0x3d, 0xde, 0x00]).as_deref(),
            Some("\u{1f600}")
        );
        assert_eq!(decode(Tag::BmpString, &[0x00, 0x41, 0x00]), None);
        assert_eq!(decode(Tag::BmpString, &[0xd8, 0x3d]), None);
    }

    #[test]
    fn teletex_is_latin1() {
        assert_eq!(
            decode(Tag::TeletexString, &[0x4a, 0xe3, 0x6f]).as_deref(),
            Some("Jão")
        );
    }

    #[test]
    fn other_types_are_ignored() {
        for tag in [
            Tag::OctetString,
            Tag::NumericString,
            Tag::VisibleString,
            Tag::Integer,
        ] {
            assert_eq!(decode(tag, b"abc"), None);
        }
    }
}
