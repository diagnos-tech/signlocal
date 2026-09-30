//! The catalog the windows read their text from (system locale).

use websign_i18n::{Catalog, Locale};

/// The catalog for the system locale.
pub fn catalog() -> Catalog {
    Catalog::new(Locale::from_system())
}
