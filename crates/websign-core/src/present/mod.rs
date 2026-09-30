//! Text rules the confirmation window, the diagnostics window and the site
//! all rely on: how an origin, a holder name, a document number and a desktop
//! caller are shown (`docs/ux.md` §4.3, §5.2, §5.5, §16).
//!
//! Locale-independent: functions return structured values; the app turns them
//! into words with `websign-i18n`.

pub mod caller;
pub mod document;
pub mod holder;
pub mod origin;
pub mod wire;
