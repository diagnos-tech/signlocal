//! Whether a listed certificate can sign right now (`docs/ux.md` §5.8).

use websign_core::CertInfo;

use super::candidate::{CertCandidate, PinMode};
use super::order::ListContext;
use super::row::{DisabledReason, RowStatus};

/// The status of a certificate that is not hidden: the first reason it
/// cannot sign, else usable.
pub(super) fn row_status(
    candidate: &CertCandidate,
    info: &CertInfo,
    context: &ListContext,
) -> RowStatus {
    match disabled_reason(candidate, info, context) {
        Some(reason) => RowStatus::Disabled(reason),
        None => RowStatus::Usable,
    }
}

fn disabled_reason(
    candidate: &CertCandidate,
    info: &CertInfo,
    context: &ListContext,
) -> Option<DisabledReason> {
    if candidate.removed {
        return Some(DisabledReason::Removed);
    }
    if context.now > info.not_after {
        return Some(DisabledReason::Expired);
    }
    if context.now < info.not_before {
        return Some(DisabledReason::NotYetValid);
    }
    if matches!(candidate.pin, PinMode::App { locked: true, .. }) {
        return Some(DisabledReason::PinLocked);
    }
    let compatible = context.accepted.is_empty()
        || context
            .accepted
            .iter()
            .any(|algorithm| candidate.algorithms.contains(algorithm));
    (!compatible).then_some(DisabledReason::Incompatible)
}
