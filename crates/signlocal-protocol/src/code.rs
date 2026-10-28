//! The verification code: a short, comparable rendering of a digest.
//!
//! The confirmation window and the website show the same code next to their
//! "Sign" buttons, so a person can check that the document the site prepared
//! is the one being signed (`docs/ux.md` §4.4). The algorithm must be
//! identical here, in the SDK (`fingerprint()`) and in every client library;
//! the vectors in `SPEC.md` §8 pin it down.

use serde::{Deserialize, Serialize};

/// Bytes of the digest the code is built from.
pub const CODE_BYTES: usize = 8;

/// Side of the identicon grid.
pub const GRID: usize = 5;

/// A digest's verification code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct VerificationCode {
    /// First 8 digest bytes as uppercase hex in 4 groups of 4, separated by
    /// single spaces: `"7F3A 9C21 E0B4 55D8"`.
    pub text: String,
    /// Identicon color, 0..=7 (`docs/ux.md` §11.1 palette `id-0` … `id-7`).
    pub color_index: u8,
    /// 25 identicon cells, row-major (`cells[row * 5 + column]`), `true` = lit.
    /// Columns 3 and 4 mirror columns 1 and 0.
    pub cells: Vec<bool>,
}

/// The verification code of `digest`, or `None` when it is shorter than
/// [`CODE_BYTES`] (no supported hash is).
pub fn verification_code(digest: &[u8]) -> Option<VerificationCode> {
    let head: &[u8; CODE_BYTES] = digest.get(..CODE_BYTES)?.try_into().ok()?;
    let groups: Vec<String> = head
        .chunks(2)
        .map(|pair| pair.iter().map(|b| format!("{b:02X}")).collect())
        .collect();
    let bits = (u16::from(head[1]) << 8) | u16::from(head[2]);
    let cells = (0..GRID * GRID)
        .map(|index| {
            let (row, column) = (index / GRID, index % GRID);
            let source = if column < 3 {
                column
            } else {
                GRID - 1 - column
            };
            bits >> (row * 3 + source) & 1 == 1
        })
        .collect();
    Some(VerificationCode {
        text: groups.join(" "),
        color_index: head[0] >> 5,
        cells,
    })
}

crate::strict::object_serde!(VerificationCode);

#[cfg(test)]
mod tests {
    use super::*;

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
        let code = verification_code(digest).expect("long enough");
        assert_eq!(code.text, text);
        assert_eq!(code.color_index, color);
        assert_eq!(rows(&code), expected_rows);
    }

    #[test]
    fn matches_the_specified_vectors() {
        check(
            &[0x7F, 0x3A, 0x9C, 0x21, 0xE0, 0xB4, 0x55, 0xD8],
            "7F3A 9C21 E0B4 55D8",
            3,
            "00100 11011 01010 10101 11011",
        );
        check(
            &[0xE3, 0xB0, 0xC4, 0x42, 0x98, 0xFC, 0x1C, 0x14],
            "E3B0 C442 98FC 1C14",
            7,
            "00100 00000 11011 00000 11011",
        );
        check(
            &[0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA],
            "BA78 16BF 8F01 CFEA",
            5,
            "01110 01010 00000 00100 11111",
        );
        check(
            &[0xCB, 0x00, 0x75, 0x3F, 0x45, 0xA3, 0x5E, 0x8B],
            "CB00 753F 45A3 5E8B",
            6,
            "10101 01110 10001 00000 00000",
        );
    }

    #[test]
    fn handles_all_zero_and_all_one_digests() {
        check(
            &[0; 32],
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
    }

    #[test]
    fn refuses_short_digests() {
        assert_eq!(verification_code(&[0; 7]), None);
        assert_eq!(verification_code(&[]), None);
    }
}
