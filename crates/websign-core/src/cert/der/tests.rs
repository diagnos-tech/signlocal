use super::*;

fn read_all(input: &[u8]) -> Result<Vec<Tlv<'_>>, DerError> {
    Reader::new(input).collect()
}

#[test]
fn reads_every_definite_length_form() {
    let mut long = vec![0x04, 0x81, 0x80];
    long.extend([7; 0x80]);
    let cases: [(&[u8], usize); 5] = [
        (&[0x04, 0x01, 0xaa], 1),
        (&long, 0x80),
        (&[0x04, 0x81, 0x01, 0xaa], 1),
        (&[0x04, 0x82, 0x00, 0x01, 0xaa], 1),
        (&[0x04, 0x84, 0x00, 0x00, 0x00, 0x01, 0xaa], 1),
    ];
    for (input, len) in cases {
        let content = Reader::new(input).read(OCTET_STRING).unwrap();
        assert_eq!(content.len(), len, "{input:02x?}");
    }
}

#[test]
fn refuses_ambiguous_lengths() {
    let cases: [(&[u8], DerError); 6] = [
        (&[0x30, 0x80, 0x00, 0x00], DerError::UnsupportedLength),
        (&[0x30, 0x85, 0, 0, 0, 0, 1, 0], DerError::UnsupportedLength),
        (&[0x30, 0xff], DerError::UnsupportedLength),
        (&[0x30, 0x81], DerError::Truncated),
        (&[0x30, 0x02, 0x00], DerError::Truncated),
        (&[0x30], DerError::Truncated),
    ];
    for (input, error) in cases {
        assert_eq!(read_all(input), Err(error), "{input:02x?}");
    }
}

#[test]
fn refuses_multi_byte_tags() {
    assert_eq!(read_all(&[0x1f, 0x81, 0x00]), Err(DerError::HighTagNumber));
}

#[test]
fn iterates_elements_and_stops_at_the_first_error() {
    let input = [0x02, 0x01, 0x05, 0x05, 0x00, 0x04, 0x05];
    let mut reader = Reader::new(&input);
    assert_eq!(
        reader.next(),
        Some(Ok(Tlv {
            tag: INTEGER,
            content: &[5]
        }))
    );
    assert_eq!(
        reader.next(),
        Some(Ok(Tlv {
            tag: 0x05,
            content: &[]
        }))
    );
    assert_eq!(reader.next(), Some(Err(DerError::Truncated)));
    assert_eq!(reader.next(), None);
}

#[test]
fn typed_reads_check_the_tag_and_optional_reads_skip_absent_fields() {
    let input = [0x02, 0x01, 0x05];
    assert_eq!(
        Reader::new(&input).read(SEQUENCE),
        Err(DerError::UnexpectedTag {
            expected: SEQUENCE,
            found: INTEGER
        })
    );
    let mut reader = Reader::new(&input);
    assert_eq!(reader.read_optional(BOOLEAN), Ok(None));
    assert_eq!(reader.read_optional(INTEGER), Ok(Some(&[5][..])));
    assert_eq!(reader.finish(), Ok(()));
}

#[test]
fn elements_must_all_have_the_expected_tag() {
    let two_ints = [0x02, 0x01, 0x01, 0x02, 0x01, 0x02];
    let values: Result<Vec<_>, _> = elements(&two_ints, INTEGER).collect();
    assert_eq!(values, Ok(vec![&[1][..], &[2][..]]));
    let mixed = [0x02, 0x01, 0x01, 0x05, 0x00];
    assert!(elements(&mixed, INTEGER).any(|e| e.is_err()));
}

#[test]
fn oid_and_optional_takes_at_most_one_extra_element() {
    let oid = [0x06, 0x03, 0x55, 0x1d, 0x20];
    assert_eq!(oid_and_optional(&oid), Ok((&[0x55, 0x1d, 0x20][..], None)));
    let with_null = [&oid[..], &[0x05, 0x00]].concat();
    let (_, info) = oid_and_optional(&with_null).unwrap();
    assert_eq!(info.map(|tlv| tlv.tag), Some(0x05));
    let with_two = [&with_null[..], &[0x05, 0x00]].concat();
    assert_eq!(oid_and_optional(&with_two), Err(DerError::TrailingData));
    assert!(oid_and_optional(&[0x02, 0x01, 0x05]).is_err());
    assert!(oid_and_optional(&[]).is_err());
}

#[test]
fn single_rejects_trailing_bytes() {
    assert_eq!(single(&[0x05, 0x00], 0x05), Ok(&[][..]));
    assert_eq!(
        single(&[0x05, 0x00, 0x00], 0x05),
        Err(DerError::TrailingData)
    );
}

#[test]
fn booleans_accept_any_non_zero_octet_as_true() {
    assert_eq!(boolean(&[0xff]), Ok(true));
    assert_eq!(boolean(&[0x01]), Ok(true));
    assert_eq!(boolean(&[0x00]), Ok(false));
    assert!(boolean(&[]).is_err());
    assert!(boolean(&[0, 0]).is_err());
}

#[test]
fn bit_strings_number_bits_from_the_top_and_pad_with_zeros() {
    let bits = bit_string(&[0x01, 0x86]).unwrap();
    let set: Vec<usize> = (0..16).filter(|&i| bits.bit(i)).collect();
    assert_eq!(set, [0, 5, 6]);
    assert_eq!(bits.whole_bytes(), None);
    assert_eq!(bit_string(&[0x00]).unwrap().whole_bytes(), Some(&[][..]));
    for bad in [&[][..], &[0x08, 0x00], &[0x01]] {
        assert!(bit_string(bad).is_err(), "{bad:02x?}");
    }
}
