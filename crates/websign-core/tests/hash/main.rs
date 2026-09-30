//! SPEC §1: `HashAlgorithm`, `DigestLengthError`.

mod check_digest;
mod constants;
mod digest;
mod digest_length_error;
mod from_str;

#[path = "../common/mod.rs"]
mod common;

use std::collections::HashSet;

use common::{FIXTURE_MESSAGE, fixture_digest, hex_decode};
use websign_core::{DigestLengthError, HashAlgorithm, UnknownAlgorithmError};

use HashAlgorithm::{Sha256, Sha384, Sha512};

fn repeated_a(count: usize) -> Vec<u8> {
    vec![b'a'; count]
}
