//! Native messaging framing: native-endian `u32` length + body (SPEC §6).

use std::io::{self, Read, Write};

use websign_protocol::framing::{FrameError, MAX_INCOMING, MAX_OUTGOING, read_frame, write_frame};
use websign_protocol::limits;

fn framed(payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_ne_bytes().to_vec();
    bytes.extend_from_slice(payload);
    bytes
}

/// Hands out at most `chunk` bytes per `read`, like a pipe.
struct Trickle<'a> {
    data: &'a [u8],
    chunk: usize,
}

impl Read for Trickle<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.chunk.min(buf.len()).min(self.data.len());
        buf[..n].copy_from_slice(&self.data[..n]);
        self.data = &self.data[n..];
        Ok(n)
    }
}

struct Failing(io::ErrorKind);

impl Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from(self.0))
    }
}

impl Write for Failing {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(self.0))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::from(self.0))
    }
}

/// Fails with `Interrupted` before every successful read.
struct Interrupting<'a> {
    inner: &'a [u8],
    interrupt_next: bool,
}

impl Read for Interrupting<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.interrupt_next {
            self.interrupt_next = false;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        self.interrupt_next = true;
        self.inner.read(buf)
    }
}

#[test]
fn limits_are_one_mebibyte() {
    assert_eq!(MAX_INCOMING, 1024 * 1024);
    assert_eq!(MAX_OUTGOING, 1024 * 1024);
    assert_eq!(limits::MAX_INCOMING_FRAME, MAX_INCOMING);
    assert_eq!(limits::MAX_OUTGOING_FRAME, MAX_OUTGOING);
}

#[test]
fn header_is_native_endian_u32() {
    let mut out = Vec::new();
    write_frame(&mut out, b"{}").unwrap();
    assert_eq!(out[..4], 2u32.to_ne_bytes());
    assert_eq!(&out[4..], b"{}");

    let mut out = Vec::new();
    write_frame(&mut out, &vec![b' '; 0x0001_0203]).unwrap();
    assert_eq!(out[..4], 0x0001_0203u32.to_ne_bytes());
    assert_eq!(out.len(), 4 + 0x0001_0203);
}

#[test]
fn reads_a_hand_built_frame() {
    let mut reader = framed(br#"{"a":1}"#);
    reader.extend_from_slice(&framed(b"{}"));
    let mut reader = reader.as_slice();
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), br#"{"a":1}"#);
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"{}");
    assert!(read_frame(&mut reader).unwrap().is_none());
}

#[test]
fn round_trips_several_frames_including_empty_and_utf8() {
    let payloads: [&[u8]; 4] = [
        br#"{"a":1}"#,
        b"",
        "{\"n\":\"ação 日本\"}".as_bytes(),
        &[0u8, 255, 10, 13],
    ];
    let mut stream = Vec::new();
    for payload in payloads {
        write_frame(&mut stream, payload).unwrap();
    }
    let mut reader = stream.as_slice();
    for payload in payloads {
        assert_eq!(read_frame(&mut reader).unwrap().unwrap(), payload);
    }
    assert!(read_frame(&mut reader).unwrap().is_none());
}

#[test]
fn clean_end_between_frames_is_none() {
    assert!(read_frame(&mut io::empty()).unwrap().is_none());
    let bytes = framed(b"x");
    let mut cursor = bytes.as_slice();
    read_frame(&mut cursor).unwrap();
    assert!(read_frame(&mut cursor).unwrap().is_none());
    assert!(read_frame(&mut cursor).unwrap().is_none());
}

#[test]
fn end_inside_the_header_is_truncated() {
    for len in 1..4 {
        let bytes = vec![1u8; len];
        let error = read_frame(&mut bytes.as_slice()).unwrap_err();
        assert!(matches!(error, FrameError::Truncated), "{len}: {error:?}");
    }
}

#[test]
fn end_inside_the_body_is_truncated() {
    let mut bytes = framed(b"0123456789");
    for cut in 4..bytes.len() {
        let error = read_frame(&mut &bytes[..cut]).unwrap_err();
        assert!(
            matches!(error, FrameError::Truncated),
            "cut {cut}: {error:?}"
        );
    }
    bytes.truncate(4);
    assert!(matches!(
        read_frame(&mut bytes.as_slice()),
        Err(FrameError::Truncated)
    ));
}

#[test]
fn announced_length_over_the_limit_is_too_large_without_reading_the_body() {
    for len in [MAX_INCOMING as u32 + 1, u32::MAX, 0x8000_0000] {
        let header = len.to_ne_bytes();
        match read_frame(&mut header.as_slice()) {
            Err(FrameError::TooLarge { len: got, max }) => {
                assert_eq!(got, len as usize);
                assert_eq!(max, MAX_INCOMING);
            }
            other => panic!("{len}: {other:?}"),
        }
    }
}

#[test]
fn a_frame_of_exactly_the_limit_is_accepted() {
    let payload = vec![b'x'; MAX_INCOMING];
    let bytes = framed(&payload);
    assert_eq!(
        read_frame(&mut bytes.as_slice()).unwrap().unwrap().len(),
        MAX_INCOMING
    );
}

#[test]
fn writing_over_the_limit_fails_and_writes_nothing() {
    let payload = vec![b'x'; MAX_OUTGOING + 1];
    let mut out = Vec::new();
    match write_frame(&mut out, &payload) {
        Err(FrameError::TooLarge { len, max }) => {
            assert_eq!(len, MAX_OUTGOING + 1);
            assert_eq!(max, MAX_OUTGOING);
        }
        other => panic!("{other:?}"),
    }
    assert!(out.is_empty());
}

#[test]
fn writing_exactly_the_limit_succeeds() {
    let mut out = Vec::new();
    write_frame(&mut out, &vec![b'x'; MAX_OUTGOING]).unwrap();
    assert_eq!(out.len(), 4 + MAX_OUTGOING);
}

#[test]
fn reads_survive_short_reads() {
    let mut stream = framed(br#"{"hello":"world"}"#);
    stream.extend_from_slice(&framed(b"{}"));
    let mut reader = Trickle {
        data: &stream,
        chunk: 1,
    };
    assert_eq!(
        read_frame(&mut reader).unwrap().unwrap(),
        br#"{"hello":"world"}"#
    );
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"{}");
    assert!(read_frame(&mut reader).unwrap().is_none());

    let mut reader = Trickle {
        data: &stream,
        chunk: 3,
    };
    assert_eq!(
        read_frame(&mut reader).unwrap().unwrap(),
        br#"{"hello":"world"}"#
    );
}

#[test]
fn truncation_is_detected_across_short_reads() {
    let stream = framed(b"0123456789");
    let mut reader = Trickle {
        data: &stream[..9],
        chunk: 2,
    };
    assert!(matches!(
        read_frame(&mut reader),
        Err(FrameError::Truncated)
    ));
}

#[test]
fn interrupted_reads_are_retried() {
    let stream = framed(b"{}");
    let mut reader = Interrupting {
        inner: &stream,
        interrupt_next: true,
    };
    assert_eq!(read_frame(&mut reader).unwrap().unwrap(), b"{}");
}

#[test]
fn other_io_errors_are_surfaced() {
    let error = read_frame(&mut Failing(io::ErrorKind::PermissionDenied)).unwrap_err();
    match error {
        FrameError::Io(io) => assert_eq!(io.kind(), io::ErrorKind::PermissionDenied),
        other => panic!("{other:?}"),
    }
    let error = write_frame(&mut Failing(io::ErrorKind::BrokenPipe), b"{}").unwrap_err();
    match error {
        FrameError::Io(io) => assert_eq!(io.kind(), io::ErrorKind::BrokenPipe),
        other => panic!("{other:?}"),
    }
}

#[test]
fn errors_have_readable_messages() {
    let too_large = FrameError::TooLarge {
        len: 2_000_000,
        max: MAX_INCOMING,
    };
    assert!(too_large.to_string().contains("2000000"));
    assert!(!FrameError::Truncated.to_string().is_empty());
}
