//! SPEC §9 (`docs/ux.md` §4.3, vectors §16.2): `present::origin::format_origin`.

mod hosts;
mod malformed;
mod robustness;
mod secure_context;
mod vectors;

#[path = "../common/mod.rs"]
mod common;

use common::Rng;
use websign_core::present::origin::{FormattedOrigin, OriginError, OriginWarning, format_origin};

use OriginWarning::{Idn, LocalIp, Localhost, PublicIp};

/// Cyrillic "а" (U+0430) in place of the Latin one: the classic look-alike.
const LOOKALIKE_UNICODE: &str = "di\u{430}gnos.health";

struct Row {
    input: &'static str,
    canonical: &'static str,
    prefix: &'static str,
    registrable: &'static str,
    port: Option<u16>,
    warning: Option<OriginWarning>,
    can_remember: bool,
}

const fn row(
    input: &'static str,
    canonical: &'static str,
    prefix: &'static str,
    registrable: &'static str,
    port: Option<u16>,
    warning: Option<OriginWarning>,
    can_remember: bool,
) -> Row {
    Row {
        input,
        canonical,
        prefix,
        registrable,
        port,
        warning,
        can_remember,
    }
}

fn ok(input: &str) -> FormattedOrigin {
    format_origin(input).unwrap_or_else(|e| panic!("{input:?} must be accepted, got {e:?}"))
}

fn check(rows: &[Row]) {
    for r in rows {
        let got = ok(r.input);
        assert_eq!(got.canonical, r.canonical, "{} canonical", r.input);
        assert_eq!(got.prefix, r.prefix, "{} prefix", r.input);
        assert_eq!(got.registrable, r.registrable, "{} registrable", r.input);
        assert_eq!(got.port, r.port, "{} port", r.input);
        assert_eq!(got.warning, r.warning, "{} warning", r.input);
        assert_eq!(got.can_remember, r.can_remember, "{} can_remember", r.input);
    }
}
