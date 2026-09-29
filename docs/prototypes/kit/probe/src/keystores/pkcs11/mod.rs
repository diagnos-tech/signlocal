//! PKCS#11 modules: p11-kit registrations, known vendor paths and modules
//! given on the command line. One keystore per loaded module.

use super::{Opened, Options};

/// Loads every PKCS#11 module that can be found and adds one keystore per module.
pub fn open(options: &Options, opened: &mut Opened) {
    let _ = (options, opened);
    todo!("pkcs11 keystore")
}
