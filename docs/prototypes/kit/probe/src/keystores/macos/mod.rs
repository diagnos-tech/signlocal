//! macOS keychains, including CryptoTokenKit smart card tokens, signing
//! through `SecKeyCreateSignature`.

use super::{Opened, Options};

/// Adds the macOS keychain source to `opened`.
pub fn open(options: &Options, opened: &mut Opened) {
    let _ = (options, opened);
    todo!("macos keystore")
}
