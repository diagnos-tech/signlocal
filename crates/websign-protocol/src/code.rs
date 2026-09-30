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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
    let _ = digest;
    todo!("SPEC.md §8")
}
