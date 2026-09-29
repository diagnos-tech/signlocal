//! EU qualified certificate statements (RFC 3739, ETSI EN 319 412-5).

/// What the qcStatements extension declares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Qualified {
    /// QcCompliance: issued as a qualified certificate under eIDAS.
    pub compliance: bool,
    /// QcSSCD: the private key lives in a qualified signature creation device.
    pub sscd: bool,
    /// QcType values, in certificate order.
    pub types: Vec<QcType>,
}

/// QcType: what the qualified certificate is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QcType {
    ESign,
    ESeal,
    Web,
}
