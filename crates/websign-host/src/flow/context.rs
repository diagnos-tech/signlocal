//! What the certificate list depends on besides the candidates.

use std::time::SystemTime;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_core::present::wire::signature_algorithm;
use websign_core::{Fingerprint, SignatureAlgorithm};
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::certs::ListContext;

use crate::ports::unix_seconds;

/// The list rules' inputs at `wall`: validity is judged at that instant and
/// dates are the computer's local ones, as in the window.
pub fn list_context(
    wall: SystemTime,
    accepted: Vec<SignatureAlgorithm>,
    requested: Option<Fingerprint>,
    last_used_here: Option<Fingerprint>,
    recent_anywhere: Vec<Fingerprint>,
) -> ListContext {
    let instant = Timestamp::try_from(wall).unwrap_or(Timestamp::UNIX_EPOCH);
    let time_zone = TimeZone::system();
    ListContext {
        today: instant.to_zoned(time_zone.clone()).date(),
        time_zone,
        now: unix_seconds(wall),
        accepted,
        last_used_here,
        recent_anywhere,
        requested,
    }
}

/// Wire algorithm names as the list rules' algorithms.
pub(crate) fn accepted_algorithms(names: &[SignatureAlgorithmName]) -> Vec<SignatureAlgorithm> {
    names.iter().copied().map(signature_algorithm).collect()
}
