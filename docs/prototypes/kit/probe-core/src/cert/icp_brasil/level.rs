//! ICP-Brasil certificate levels.

use std::fmt;

/// Certificate type from the ICP-Brasil policy OID (DOC-ICP-04): `A` for
/// signature, `S` for confidentiality, `T` for time stamping; the digit is
/// the security level, which grows with key protection (A1 keys live in a
/// file, A3 keys in a token or smart card).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IcpLevel {
    A1,
    A2,
    A3,
    A4,
    S1,
    S2,
    S3,
    S4,
    T3,
    T4,
    /// A policy arc this crate does not know yet.
    Other(u32),
}

impl IcpLevel {
    /// The level of the first policy under `prefix` (the ICP-Brasil policy
    /// arc). An arc too large for [`IcpLevel::Other`] leaves the level
    /// unknown rather than guessing it from a later policy.
    pub(super) fn from_policies(policies: &[String], prefix: &str) -> Option<Self> {
        let first = policies
            .iter()
            .find_map(|policy| policy.strip_prefix(prefix))?;
        let arc = first.split('.').next()?;
        arc.parse::<u32>().ok().map(Self::from_arc)
    }

    fn from_arc(arc: u32) -> Self {
        match arc {
            1 => Self::A1,
            2 => Self::A2,
            3 => Self::A3,
            4 => Self::A4,
            101 => Self::S1,
            102 => Self::S2,
            103 => Self::S3,
            104 => Self::S4,
            303 => Self::T3,
            304 => Self::T4,
            other => Self::Other(other),
        }
    }
}

impl fmt::Display for IcpLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::A4 => "A4",
            Self::S1 => "S1",
            Self::S2 => "S2",
            Self::S3 => "S3",
            Self::S4 => "S4",
            Self::T3 => "T3",
            Self::T4 => "T4",
            Self::Other(arc) => return write!(f, "ICP-Brasil ({arc})"),
        };
        f.write_str(name)
    }
}
