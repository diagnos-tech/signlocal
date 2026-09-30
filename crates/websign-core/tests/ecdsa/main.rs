//! SPEC §4: `Curve`, `der_to_raw`, `raw_to_der`.

mod curve;
mod der_to_raw_rejected;
mod der_to_raw_valid;
mod raw_to_der;
mod round_trip;

#[path = "../common/mod.rs"]
mod common;

use std::collections::HashSet;

use common::{
    Rng, counting_bytes, der_ecdsa, ecdsa_vectors, hex_decode, left_pad, parse_strict_ecdsa_der,
};
use websign_core::ecdsa::{Curve, EcdsaEncodingError, der_to_raw, raw_to_der};

use Curve::{P256, P384, P521};
use EcdsaEncodingError::{IntegerTooLarge, Malformed, WrongLength};

const ALL_CURVES: [Curve; 3] = [P256, P384, P521];

/// `r || s` from big-endian magnitudes, each left-padded to the field size.
fn raw_of(curve: Curve, r: &[u8], s: &[u8]) -> Vec<u8> {
    let mut out = left_pad(r, curve.field_len());
    out.extend(left_pad(s, curve.field_len()));
    out
}

fn zeros(count: usize) -> Vec<u8> {
    vec![0; count]
}

fn concat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}
