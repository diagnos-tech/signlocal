//! SPEC §5: `Fingerprint`, `ParseFingerprintError`.

mod construction;
mod derived_traits;
mod display_and_debug;
mod from_str;
mod to_hex;

#[path = "../common/mod.rs"]
mod common;

use std::collections::{BTreeSet, HashSet};

use common::{cert, cert_names, counting_bytes, fixture_fingerprint, hex_decode};
use websign_core::{Fingerprint, ParseFingerprintError};

const ABC_HEX: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY_HEX: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn bytes_32(start: u8) -> [u8; 32] {
    counting_bytes(start, 32).try_into().unwrap()
}

fn from_hex(hex: &str) -> Fingerprint {
    Fingerprint::from_bytes(hex_decode(hex).try_into().expect("32 bytes"))
}

/// `00:01:02:...:1F` for the bytes 0..32.
fn counting_display() -> String {
    (0..32)
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}
