//! EU qualified certificate statements (RFC 3739, ETSI EN 319 412-5).

use super::der::{self, DerError, OBJECT_IDENTIFIER, SEQUENCE, Tlv};
use super::oid::{QC_COMPLIANCE, QC_SSCD, QC_TYPE, QC_TYPE_ESEAL, QC_TYPE_ESIGN, QC_TYPE_WEB};

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

impl Qualified {
    /// Decodes the value of a qcStatements extension:
    /// `SEQUENCE OF SEQUENCE { statementId OID, statementInfo ANY OPTIONAL }`.
    ///
    /// Statements this crate does not know are ignored, so new eIDAS
    /// statements never make a certificate unreadable. The types of every
    /// QcType statement are collected in certificate order; one without
    /// statementInfo contributes none.
    pub(super) fn from_extension(extension_value: &[u8]) -> Result<Self, DerError> {
        let mut qualified = Self::default();
        for statement in der::elements(der::single(extension_value, SEQUENCE)?, SEQUENCE) {
            match der::oid_and_optional(statement?)? {
                (QC_COMPLIANCE, _) => qualified.compliance = true,
                (QC_SSCD, _) => qualified.sscd = true,
                (QC_TYPE, Some(info)) => qualified.types.extend(qc_types(info)?),
                _ => {}
            }
        }
        Ok(qualified)
    }
}

/// The recognised entries of a QcType `SEQUENCE OF OBJECT IDENTIFIER`.
fn qc_types(info: Tlv<'_>) -> Result<Vec<QcType>, DerError> {
    if info.tag != SEQUENCE {
        return Err(DerError::UnexpectedTag {
            expected: SEQUENCE,
            found: info.tag,
        });
    }
    let mut types = Vec::new();
    for oid in der::elements(info.content, OBJECT_IDENTIFIER) {
        match oid? {
            QC_TYPE_ESIGN => types.push(QcType::ESign),
            QC_TYPE_ESEAL => types.push(QcType::ESeal),
            QC_TYPE_WEB => types.push(QcType::Web),
            _ => {}
        }
    }
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut out = vec![tag, u8::try_from(content.len()).unwrap()];
        out.extend_from_slice(content);
        out
    }

    /// OID `0.4.0.1862.1.<arcs>` as a TLV.
    fn qc_oid(arcs: &[u8]) -> Vec<u8> {
        tlv(0x06, &[&[0x04, 0x00, 0x8e, 0x46, 0x01], arcs].concat())
    }

    fn statement(id: &[u8], info: Option<Vec<u8>>) -> Vec<u8> {
        tlv(0x30, &[qc_oid(id), info.unwrap_or_default()].concat())
    }

    fn extension(statements: &[Vec<u8>]) -> Vec<u8> {
        tlv(0x30, &statements.concat())
    }

    fn types(oids: &[Vec<u8>]) -> Option<Vec<u8>> {
        Some(tlv(0x30, &oids.concat()))
    }

    #[test]
    fn types_of_every_qc_type_statement_are_collected_in_order() {
        let ext = extension(&[
            statement(
                &[0x06],
                types(&[qc_oid(&[0x06, 0x03]), qc_oid(&[0x06, 0x09])]),
            ),
            statement(&[0x01], None),
            statement(&[0x06], None),
            statement(
                &[0x06],
                types(&[qc_oid(&[0x06, 0x01]), qc_oid(&[0x06, 0x03])]),
            ),
        ]);
        let q = Qualified::from_extension(&ext).unwrap();
        assert!(q.compliance);
        assert_eq!(q.types, [QcType::Web, QcType::ESign, QcType::Web]);
    }

    #[test]
    fn info_of_other_statements_is_not_inspected() {
        let ext = extension(&[
            statement(&[0x01], Some(tlv(0x04, &[1]))),
            statement(&[0x63], Some(tlv(0x05, &[]))),
        ]);
        assert!(Qualified::from_extension(&ext).unwrap().compliance);
    }

    #[test]
    fn malformed_structures_are_errors() {
        let not_a_sequence = extension(&[statement(&[0x06], Some(tlv(0x04, &[1])))]);
        let not_oids = extension(&[statement(&[0x06], types(&[tlv(0x02, &[1])]))]);
        let no_oid_first = extension(&[tlv(0x30, &tlv(0x02, &[1]))]);
        for bad in [not_a_sequence, not_oids, no_oid_first] {
            assert!(Qualified::from_extension(&bad).is_err(), "{bad:02x?}");
        }
        assert!(Qualified::from_extension(&tlv(0x04, &[])).is_err());
        assert!(Qualified::from_extension(&[]).is_err());
    }
}
