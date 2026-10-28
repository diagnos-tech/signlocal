//! Test your program without the signlocal app (feature `testing`).
//!
//! [`FakeApp`] is an in-process stand-in for `signlocal connect`: it speaks the
//! real protocol, so the code under test runs unchanged. Enable it for tests
//! only:
//!
//! ```toml
//! [dev-dependencies]
//! signlocal-client = { version = "0.1", features = ["testing"] }
//! ```

mod app;
mod sample;
mod serve;

pub use app::{FakeApp, FakeAppBuilder};
pub use sample::sample_certificate;
