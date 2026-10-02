//! SPEC §4 and §4.1: `Curve` with the Brainpool curves, OID mapping, and the
//! signature encodings on curves with 32, 48 and 64 byte fields.

mod curve;
mod encodings;
mod openssl;

#[path = "../common/mod.rs"]
mod common;

use std::collections::HashSet;

use common::{
    BRAINPOOL_R1_KEYS, BRAINPOOL_TWISTED, Rng, brainpool_ecdsa_vectors, counting_bytes, der_ecdsa,
    left_pad, parse_strict_ecdsa_der,
};
use websign_core::ecdsa::{Curve, EcdsaEncodingError, der_to_raw, raw_to_der};

use Curve::{BrainpoolP256r1, BrainpoolP384r1, BrainpoolP512r1, P256, P384, P521};

const BRAINPOOL: [Curve; 3] = [BrainpoolP256r1, BrainpoolP384r1, BrainpoolP512r1];
const ALL: [Curve; 6] = [
    P256,
    P384,
    P521,
    BrainpoolP256r1,
    BrainpoolP384r1,
    BrainpoolP512r1,
];
