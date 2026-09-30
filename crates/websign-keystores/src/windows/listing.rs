//! Listing `CurrentUser\MY`: every certificate whose `CERT_KEY_PROV_INFO`
//! points to a private key, described from metadata only.

use websign_core::SourceKind;

use super::NAME;
use super::hardware::HardwareProbe;
use super::key_info::KeyLocation;
use super::reader;
use super::store::CertStore;
use crate::{FoundKey, PinPrompt};
use log::trace;

/// Keys of the store, in store order. No key is opened for signing and no
/// PIN is asked: the OS shows its own PIN dialog when signing.
pub fn keys(store: &CertStore) -> Vec<FoundKey> {
    let mut hardware = HardwareProbe::default();
    let mut found = Vec::new();
    trace!("enumerating CurrentUser\\MY (key metadata only)");
    for (index, cert) in store.certificates().enumerate() {
        let (Some(location), Some(thumbprint)) = (KeyLocation::of(&cert), cert.thumbprint()) else {
            trace!("certificate #{index}: no private key, skipped");
            continue;
        };
        trace!("certificate #{index}: key in {}", location.describe());
        let in_hardware = hardware.is_hardware(&location);
        let device = if in_hardware == Some(true) {
            reader::device_link(&location)
        } else {
            None
        };
        found.push(FoundKey {
            cert_der: cert.der().to_vec(),
            keystore: NAME.to_owned(),
            kind: SourceKind::System,
            provider: location.describe(),
            hardware: in_hardware,
            locator: thumbprint.to_hex(),
            pin: PinPrompt::System,
            device,
        });
    }
    found
}
