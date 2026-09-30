//! Builders shared by the integration tests. Every default is the boring
//! case (a signing certificate valid for years), so a test names only the one
//! field it is about.
#![allow(dead_code, unused_imports)]

pub mod cert;
pub mod session;

pub use cert::*;
pub use session::*;
