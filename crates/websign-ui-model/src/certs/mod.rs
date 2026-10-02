//! The certificate list (`docs/ux.md` §5): which certificates appear, how each
//! row reads, their order and the initial selection.

mod badge;
mod build_row;
mod candidate;
mod filter;
mod hide;
mod location;
mod merge;
mod order;
mod row;
#[cfg(test)]
mod row_tests;
mod status;
mod text;
mod validity;

pub use badge::{Badge, badge};
pub use candidate::{CertCandidate, DeviceLabel, KeyPath, KeySource, PinMode};
pub use filter::{FILTER_THRESHOLD, matches_filter};
pub use location::{Location, Place, location};
pub use order::{ListContext, build_cert_list};
pub use row::{CertList, CertRow, DisabledReason, HiddenReason, RowStatus};
pub use validity::{Tone, ValidityLabel, validity_label};
