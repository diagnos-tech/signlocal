//! Windows certificate store (`CurrentUser\MY`), signing through CNG (NCrypt)
//! or legacy CAPI, whichever the key's provider speaks.
//!
//! Listing reads certificate properties and provider metadata only, so it
//! never prompts. Keys are acquired with `PREFER_NCRYPT` by default so that
//! keys in Microsoft's CSPs sign through CNG (which adds RSASSA-PSS), with a
//! per-key fallback to plain CAPI for CSPs that refuse the bridge. The OS or
//! the middleware shows its own PIN dialog, owned by the caller's window.

mod acquire;
mod aes_reopen;
mod capabilities;
mod capi;
mod cert_context;
mod chain;
mod errors;
mod fallback;
mod handles;
mod hardware;
mod key_info;
mod keystore;
mod listing;
mod ncrypt;
mod reader;
mod reader_query;
mod request;
mod status_names;
mod store;
mod thumbprint;
mod window;

use super::{Opened, Options, SourceFailure};
use keystore::WindowsKeystore;
use store::CertStore;

/// [`crate::Keystore::name`] of this source.
const NAME: &str = "windows";

/// Room for a 16384-bit RSA signature, the largest key CNG and CAPI create.
const MAX_SIGNATURE_LEN: usize = 2048;

/// Adds the Windows store to `opened`, or records why it could not open.
pub fn open(options: &Options, opened: &mut Opened) {
    match CertStore::open_current_user_my() {
        Ok(store) => opened
            .keystores
            .push(Box::new(WindowsKeystore::new(store, options))),
        Err(error) => opened.failures.push(SourceFailure {
            source: NAME.to_owned(),
            error: error.to_string(),
        }),
    }
}
