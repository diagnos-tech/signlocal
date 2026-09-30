//! The verification code of SPEC §8 (the same table lives in the SDK and client libraries).

use serde_json::json;
use websign_protocol::code::{CODE_BYTES, GRID};
use websign_protocol::{VerificationCode, verification_code};

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn rows(code: &VerificationCode) -> String {
    code.cells
        .chunks(GRID)
        .map(|row| {
            row.iter()
                .map(|&lit| if lit { '1' } else { '0' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn check(digest: &[u8], text: &str, color: u8, expected_rows: &str) {
    let code = verification_code(digest).expect("digest is long enough");
    assert_eq!(code.text, text);
    assert_eq!(code.color_index, color);
    assert_eq!(rows(&code), expected_rows);
}

#[test]
fn spec_table_vectors() {
    check(
        &hex("7F3A9C21E0B455D8"),
        "7F3A 9C21 E0B4 55D8",
        3,
        "00100 11011 01010 10101 11011",
    );
    check(
        &hex("E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"),
        "E3B0 C442 98FC 1C14",
        7,
        "00100 00000 11011 00000 11011",
    );
    check(
        &hex("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"),
        "BA78 16BF 8F01 CFEA",
        5,
        "01110 01010 00000 00100 11111",
    );
    check(
        &[0u8; 32],
        "0000 0000 0000 0000",
        0,
        "00000 00000 00000 00000 00000",
    );
    check(
        &[0xFF; 32],
        "FFFF FFFF FFFF FFFF",
        7,
        "11111 11111 11111 11111 11111",
    );
    check(
        &hex(
            "CB00753F45A35E8BB5A03D699AC65007272C32AB0EDED1631A8B605A43FF5BED8086072BA1E7CC2358BAECA134C825A7",
        ),
        "CB00 753F 45A3 5E8B",
        6,
        "10101 01110 10001 00000 00000",
    );
}

#[test]
fn shorter_than_eight_bytes_is_none() {
    for len in 0..CODE_BYTES {
        assert!(verification_code(&vec![0xAB; len]).is_none(), "{len}");
    }
    assert!(verification_code(&[0xAB; CODE_BYTES]).is_some());
}

#[test]
fn constants() {
    assert_eq!(CODE_BYTES, 8);
    assert_eq!(GRID, 5);
}

#[test]
fn shape_is_25_cells_and_upper_hex_groups() {
    let code = verification_code(&hex("0123456789abcdef")).unwrap();
    assert_eq!(code.cells.len(), 25);
    assert_eq!(code.text, "0123 4567 89AB CDEF");
    assert!(code.color_index < 8);
}

#[test]
fn only_the_first_eight_bytes_matter() {
    let mut a = hex("7F3A9C21E0B455D8").to_vec();
    let short = verification_code(&a).unwrap();
    a.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(verification_code(&a).unwrap(), short);
    a[8] = 0xEE;
    assert_eq!(verification_code(&a).unwrap(), short);
}

#[test]
fn grid_is_mirrored_left_right() {
    for seed in 0..64u8 {
        let digest: Vec<u8> = (0..8u8)
            .map(|i| seed.wrapping_mul(31).wrapping_add(i.wrapping_mul(57)))
            .collect();
        let code = verification_code(&digest).unwrap();
        for row in code.cells.chunks(GRID) {
            assert_eq!(row[3], row[1]);
            assert_eq!(row[4], row[0]);
        }
    }
}

#[test]
fn color_uses_only_the_top_three_bits_of_the_first_byte() {
    for first in 0..=255u8 {
        let code = verification_code(&[first, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(code.color_index, first >> 5);
    }
}

#[test]
fn cells_ignore_the_top_bit_of_the_second_byte_and_the_first_byte() {
    let base = verification_code(&[0x00, 0x2A, 0x5C, 1, 2, 3, 4, 5]).unwrap();
    let high_bit = verification_code(&[0x00, 0xAA, 0x5C, 1, 2, 3, 4, 5]).unwrap();
    assert_eq!(base.cells, high_bit.cells);
    let other_first = verification_code(&[0x1F, 0x2A, 0x5C, 1, 2, 3, 4, 5]).unwrap();
    assert_eq!(base.cells, other_first.cells);
}

/// Straight transcription of the SPEC pseudo-code.
fn reference(digest: &[u8]) -> Option<(String, u8, Vec<bool>)> {
    let b = digest.get(..8)?;
    let text = b
        .chunks(2)
        .map(|p| format!("{:02X}{:02X}", p[0], p[1]))
        .collect::<Vec<_>>()
        .join(" ");
    let bits = (u32::from(b[1]) << 8) | u32::from(b[2]);
    let mut cells = vec![false; 25];
    for r in 0..5 {
        for c in 0..3 {
            let lit = bits >> (r * 3 + c) & 1 == 1;
            cells[r * 5 + c] = lit;
            if c == 1 {
                cells[r * 5 + 3] = lit;
            }
            if c == 0 {
                cells[r * 5 + 4] = lit;
            }
        }
    }
    Some((text, b[0] >> 5, cells))
}

#[test]
fn matches_the_reference_transcription_on_many_digests() {
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..2000 {
        let digest: Vec<u8> = (0..32)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 24) as u8
            })
            .collect();
        let (text, color, cells) = reference(&digest).unwrap();
        let code = verification_code(&digest).unwrap();
        assert_eq!(
            (code.text, code.color_index, code.cells),
            (text, color, cells)
        );
    }
}

#[test]
fn json_shape_is_camel_case() {
    let code = verification_code(&hex("7F3A9C21E0B455D8")).unwrap();
    let value = serde_json::to_value(&code).unwrap();
    assert_eq!(value["text"], json!("7F3A 9C21 E0B4 55D8"));
    assert_eq!(value["colorIndex"], json!(3));
    assert_eq!(value["cells"].as_array().unwrap().len(), 25);
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert_eq!(
        serde_json::from_value::<VerificationCode>(value).unwrap(),
        code
    );
}

#[test]
fn json_parsing_is_strict() {
    let extra = json!({ "text": "0000 0000 0000 0000", "colorIndex": 0, "cells": [], "extra": 1 });
    assert!(serde_json::from_value::<VerificationCode>(extra).is_err());
    let missing = json!({ "text": "0000 0000 0000 0000", "color_index": 0, "cells": [] });
    assert!(serde_json::from_value::<VerificationCode>(missing).is_err());
}
