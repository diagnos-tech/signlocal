//! System-wide manifest locations on Linux, written by the deb/rpm
//! post-install step (`websign register --scope system`, as root).
//!
//! The Firefox Snap's WebExtensions portal reads only these; distro Chrome and
//! Chromium read them too. Paths follow each browser's documented
//! system locations (`docs/research/native-messaging.md` §3).

use std::path::Path;

use crate::browsers::Browser;
use crate::destination::Target;

/// System-wide targets for `browsers`, with `lib_dirs` the distro's library
/// roots (`/usr/lib`, `/usr/lib64`).
pub fn targets(browsers: &[Browser], lib_dirs: &[&Path]) -> Vec<Target> {
    let _ = (browsers, lib_dirs);
    todo!("SPEC.md §6")
}
