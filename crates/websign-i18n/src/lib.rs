//! Translated text for the app, generated key constants, plural rules and
//! date formatting (`docs/architecture/i18n.md`).
//!
//! Messages live in `i18n/<locale>.toml` at the repository root; `en` is the
//! reference that defines the keys. The build script turns every key into a
//! constant in [`k`], so a key that does not exist does not compile:
//!
//! ```ignore
//! let catalog = Catalog::new(Locale::from_system());
//! let title = catalog.tr(k::CONFIRM_WINDOW_TITLE).arg("site", "app.example.com");
//! let expiry = catalog.plural(k::CERT_EXPIRES_IN, 23);
//! ```
//!
//! Formatting is a small pure function per locale; no ICU, no runtime data.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod catalog;
pub mod check;
pub mod dates;
mod locale;
mod message;
pub mod plural;

include!(concat!(env!("OUT_DIR"), "/keys.rs"));

pub use catalog::Catalog;
pub use locale::Locale;
pub use message::Message;

/// A message key (`"confirm.window_title"`). Only [`k`] creates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(pub(crate) &'static str);

/// A plural message key (`"cert.expires_in"`). Only [`k`] creates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluralKey(pub(crate) &'static str);

impl Key {
    /// The dotted path.
    pub const fn path(self) -> &'static str {
        self.0
    }
}

impl PluralKey {
    /// The dotted path.
    pub const fn path(self) -> &'static str {
        self.0
    }
}
