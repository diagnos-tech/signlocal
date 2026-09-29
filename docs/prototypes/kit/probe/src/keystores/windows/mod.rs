//! Windows certificate store (`CurrentUser\MY`), signing through CNG (NCrypt)
//! or legacy CAPI, whichever the key's provider speaks.

use super::{Opened, Options};

/// Adds the Windows store to `opened`.
pub fn open(options: &Options, opened: &mut Opened) {
    let _ = (options, opened);
    todo!("windows keystore")
}
