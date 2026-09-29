//! A tolerant reader for the DER that certificates are made of.
//!
//! Certificates in the field are not always strict DER: CAs and smart-card
//! middleware have shipped non-minimal lengths and integers, explicit DEFAULT
//! values, BER booleans and unsorted SETs. None of that changes what the
//! summary shows, and a certificate the user owns must not vanish from the
//! list over it, so the reader accepts all of it. What it refuses is anything
//! that makes the structure ambiguous: indefinite or truncated lengths, and
//! bytes left over where a value should end.
//!
//! Written by hand because `x509-cert` and the `der`/`const-oid` crates under
//! it reject valid certificates: they cannot even skip a UniversalString,
//! their `DateTime` stops at 1970 and their OID arcs stop at `u32` (so any
//! `2.25` UUID OID, even in an unknown extension, fails the whole decode).

#[cfg(test)]
mod tests;
mod values;

pub(super) use values::{BitString, bit_string, boolean};

pub(super) const BOOLEAN: u8 = 0x01;
pub(super) const INTEGER: u8 = 0x02;
pub(super) const BIT_STRING: u8 = 0x03;
pub(super) const OCTET_STRING: u8 = 0x04;
pub(super) const OBJECT_IDENTIFIER: u8 = 0x06;
pub(super) const UTF8_STRING: u8 = 0x0c;
pub(super) const PRINTABLE_STRING: u8 = 0x13;
pub(super) const TELETEX_STRING: u8 = 0x14;
pub(super) const IA5_STRING: u8 = 0x16;
pub(super) const UTC_TIME: u8 = 0x17;
pub(super) const GENERALIZED_TIME: u8 = 0x18;
pub(super) const BMP_STRING: u8 = 0x1e;
pub(super) const SEQUENCE: u8 = 0x30;
pub(super) const SET: u8 = 0x31;

/// Tag of a context-specific constructed field, `[n]` in ASN.1 (`n` < 31).
pub(super) const fn explicit(n: u8) -> u8 {
    0xa0 | n
}

/// Tag of a context-specific primitive field (`[n] IMPLICIT` over a primitive type).
pub(super) const fn implicit(n: u8) -> u8 {
    0x80 | n
}

/// Why bytes could not be read as the expected DER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(super) enum DerError {
    #[error("value runs past the end of its container")]
    Truncated,
    #[error("indefinite or oversized length")]
    UnsupportedLength,
    #[error("multi-byte tags are not used in certificates")]
    HighTagNumber,
    #[error("expected tag {expected:#04x}, found {found:#04x}")]
    UnexpectedTag { expected: u8, found: u8 },
    #[error("unexpected bytes after the value")]
    TrailingData,
    #[error("invalid {0}")]
    Invalid(&'static str),
}

/// One tag-length-value, borrowed from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Tlv<'a> {
    /// The identifier octet (class, constructed bit and tag number).
    pub tag: u8,
    pub content: &'a [u8],
}

/// Walks consecutive TLVs, such as the fields of a SEQUENCE or the elements
/// of a SEQUENCE OF. As an iterator it yields each element, stopping after
/// the first error.
#[derive(Debug, Clone)]
pub(super) struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(super) fn new(input: &'a [u8]) -> Self {
        Self { rest: input }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    pub(super) fn peek_tag(&self) -> Option<u8> {
        self.rest.first().copied()
    }

    pub(super) fn read_any(&mut self) -> Result<Tlv<'a>, DerError> {
        let (&tag, rest) = self.rest.split_first().ok_or(DerError::Truncated)?;
        if tag & 0x1f == 0x1f {
            return Err(DerError::HighTagNumber);
        }
        let (len, rest) = read_length(rest)?;
        let (content, rest) = rest.split_at_checked(len).ok_or(DerError::Truncated)?;
        self.rest = rest;
        Ok(Tlv { tag, content })
    }

    /// Reads the next TLV, which must have tag `tag`, and returns its content.
    pub(super) fn read(&mut self, tag: u8) -> Result<&'a [u8], DerError> {
        let tlv = self.read_any()?;
        if tlv.tag == tag {
            Ok(tlv.content)
        } else {
            Err(DerError::UnexpectedTag {
                expected: tag,
                found: tlv.tag,
            })
        }
    }

    /// Reads the next TLV only if it has tag `tag` (an OPTIONAL or DEFAULT field).
    pub(super) fn read_optional(&mut self, tag: u8) -> Result<Option<&'a [u8]>, DerError> {
        if self.peek_tag() == Some(tag) {
            self.read(tag).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Fails if anything is left, so a structure cannot hide extra fields.
    pub(super) fn finish(&self) -> Result<(), DerError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(DerError::TrailingData)
        }
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<Tlv<'a>, DerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        let item = self.read_any();
        if item.is_err() {
            self.rest = &[];
        }
        Some(item)
    }
}

/// The content of `input`, which must be exactly one TLV with tag `tag`.
pub(super) fn single(input: &[u8], tag: u8) -> Result<&[u8], DerError> {
    let mut reader = Reader::new(input);
    let content = reader.read(tag)?;
    reader.finish()?;
    Ok(content)
}

/// The contents of the elements of a SEQUENCE OF or SET OF, each of which
/// must have tag `tag`.
pub(super) fn elements(content: &[u8], tag: u8) -> impl Iterator<Item = Result<&[u8], DerError>> {
    Reader::new(content).map(move |element| {
        let element = element?;
        if element.tag == tag {
            Ok(element.content)
        } else {
            Err(DerError::UnexpectedTag {
                expected: tag,
                found: element.tag,
            })
        }
    })
}

/// Splits the content of `SEQUENCE { OBJECT IDENTIFIER, ANY OPTIONAL }`, the
/// shape of AlgorithmIdentifier, PolicyInformation and QCStatement. Anything
/// after the optional element is an error.
pub(super) fn oid_and_optional(content: &[u8]) -> Result<(&[u8], Option<Tlv<'_>>), DerError> {
    let mut fields = Reader::new(content);
    let oid = fields.read(OBJECT_IDENTIFIER)?;
    let optional = if fields.is_empty() {
        None
    } else {
        Some(fields.read_any()?)
    };
    fields.finish()?;
    Ok((oid, optional))
}

/// Short form, or long form with one to four length octets. Non-minimal
/// long forms are accepted; the indefinite form (`80`) is not, since a
/// certificate is always signed over definite lengths.
fn read_length(input: &[u8]) -> Result<(usize, &[u8]), DerError> {
    let (&first, rest) = input.split_first().ok_or(DerError::Truncated)?;
    if first < 0x80 {
        return Ok((usize::from(first), rest));
    }
    let count = usize::from(first & 0x7f);
    if !(1..=4).contains(&count) {
        return Err(DerError::UnsupportedLength);
    }
    let (octets, rest) = rest.split_at_checked(count).ok_or(DerError::Truncated)?;
    // At most four octets, so the value always fits a u32.
    let len = octets
        .iter()
        .fold(0u32, |len, &octet| (len << 8) | u32::from(octet));
    let len = usize::try_from(len).map_err(|_| DerError::UnsupportedLength)?;
    Ok((len, rest))
}
