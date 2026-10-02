use super::*;

fn framed(payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_ne_bytes().to_vec();
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn header_uses_native_byte_order() {
    let mut out = Vec::new();
    write_frame(&mut out, b"{}").unwrap();
    assert_eq!(out[..4], 2u32.to_ne_bytes());
    assert_eq!(&out[4..], b"{}");
}

#[test]
fn round_trips_several_frames_in_order() {
    let mut stream = Vec::new();
    write_frame(&mut stream, br#"{"a":1}"#).unwrap();
    write_frame(&mut stream, b"").unwrap();
    write_frame(&mut stream, "{\"n\":\"ação\"}".as_bytes()).unwrap();

    let mut reader = stream.as_slice();
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), br#"{"a":1}"#);
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"");
    assert_eq!(
        read_frame(&mut reader).unwrap().unwrap(),
        "{\"n\":\"ação\"}".as_bytes()
    );
    assert!(read_frame(&mut reader).unwrap().is_none());
}

#[test]
fn empty_stream_is_a_clean_end() {
    assert!(read_frame(&mut io::empty()).unwrap().is_none());
}

#[test]
fn end_inside_the_header_is_truncation() {
    for cut in 1..4 {
        let bytes = framed(b"{}");
        let mut reader = &bytes[..cut];
        assert!(matches!(
            read_frame(&mut reader),
            Err(FrameError::Truncated)
        ));
    }
}

#[test]
fn end_inside_the_body_is_truncation() {
    let bytes = framed(b"{\"a\":1}");
    let mut reader = &bytes[..bytes.len() - 1];
    assert!(matches!(
        read_frame(&mut reader),
        Err(FrameError::Truncated)
    ));
}

#[test]
fn incoming_limit_is_inclusive_and_checked_before_allocating() {
    let mut huge = ((MAX_INCOMING + 1) as u32).to_ne_bytes().to_vec();
    // No body follows: the limit must trip before any read of the body.
    huge.extend_from_slice(b"x");
    assert!(matches!(
        read_frame(&mut huge.as_slice()),
        Err(FrameError::TooLarge { len, max }) if len == MAX_INCOMING + 1 && max == MAX_INCOMING
    ));

    let at_limit = framed(&vec![b' '; MAX_INCOMING]);
    assert_eq!(
        read_frame(&mut at_limit.as_slice()).unwrap().unwrap().len(),
        MAX_INCOMING
    );
}

#[test]
fn a_four_gigabyte_announcement_is_rejected_without_allocating() {
    let header = u32::MAX.to_ne_bytes();
    assert!(matches!(
        read_frame(&mut header.as_slice()),
        Err(FrameError::TooLarge { .. })
    ));
}

#[test]
fn outgoing_limit_is_one_mebibyte() {
    let mut out = Vec::new();
    write_frame(&mut out, &vec![b' '; MAX_OUTGOING]).unwrap();
    let mut refused = Vec::new();
    assert!(matches!(
        write_frame(&mut refused, &vec![b' '; MAX_OUTGOING + 1]),
        Err(FrameError::TooLarge { .. })
    ));
    assert!(refused.is_empty(), "nothing may be written on refusal");
}

#[test]
fn short_reads_are_reassembled() {
    struct OneByte<'a>(&'a [u8]);
    impl Read for OneByte<'_> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let Some((&first, rest)) = self.0.split_first() else {
                return Ok(0);
            };
            buf[0] = first;
            self.0 = rest;
            Ok(1)
        }
    }
    let bytes = framed(br#"{"ok":true}"#);
    let frame = read_frame(&mut OneByte(&bytes)).unwrap().unwrap();
    assert_eq!(frame, br#"{"ok":true}"#);
}
