//! Fakes and helpers shared by the integration tests. Nothing here knows how
//! the engine works: it feeds events in and records what comes out.
#![allow(dead_code)]

pub mod certs;
pub mod effects;
pub mod fakes;
pub mod flows;
pub mod harness;
pub mod out;
pub mod stores;
pub mod wire;

#[allow(unused_imports)]
pub use certs::{Cert, snapshot};
#[allow(unused_imports)]
pub use harness::Harness;
#[allow(unused_imports)]
pub use out::Out;

/// The origin most tests call from.
pub const ORIGIN: &str = "https://app.example.com";
/// Another secure origin, for tests with two callers.
pub const OTHER_ORIGIN: &str = "https://other.example.org";
