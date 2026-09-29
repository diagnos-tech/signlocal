//! EU qualified certificate statements (RFC 3739, ETSI EN 319 412-5).

use der::Decode;
use der::Reader;
use der::asn1::{AnyRef, ObjectIdentifier};

use super::asn1::oid_with_optional_info;

const QC_COMPLIANCE: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.1");
const QC_SSCD: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.4");
const QC_TYPE: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.6");
const QC_TYPE_ESIGN: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.6.1");
const QC_TYPE_ESEAL: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.6.2");
const QC_TYPE_WEB: ObjectIdentifier = ObjectIdentifier::new_unwrap("0.4.0.1862.1.6.3");

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
    /// statements never make a certificate unreadable.
    ///
    /// SPEC: an extension with no statements at all still yields a
    /// `Qualified` (all false, no types), because the extension exists.
    pub(super) fn from_extension(extension_value: &[u8]) -> der::Result<Self> {
        let mut qualified = Self::default();
        for statement in Vec::<AnyRef>::from_der(extension_value)? {
            let (id, info) = oid_with_optional_info(statement)?;
            if id == QC_COMPLIANCE {
                qualified.compliance = true;
            } else if id == QC_SSCD {
                qualified.sscd = true;
            } else if id == QC_TYPE
                && let Some(info) = info
            {
                qualified.types.extend(qc_types(info)?);
            }
        }
        Ok(qualified)
    }
}

/// The recognised entries of a QcType `SEQUENCE OF OBJECT IDENTIFIER`.
fn qc_types(info: AnyRef<'_>) -> der::Result<Vec<QcType>> {
    let oids = info.sequence(|reader| {
        let mut oids = Vec::new();
        while !reader.is_finished() {
            oids.push(ObjectIdentifier::decode(reader)?);
        }
        Ok::<_, der::Error>(oids)
    })?;
    Ok(oids
        .into_iter()
        .filter_map(|oid| match oid {
            QC_TYPE_ESIGN => Some(QcType::ESign),
            QC_TYPE_ESEAL => Some(QcType::ESeal),
            QC_TYPE_WEB => Some(QcType::Web),
            _ => None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut out = vec![tag, content.len() as u8];
        out.extend_from_slice(content);
        out
    }

    /// OID `0.4.0.1862.1.<last>` (and optionally one more arc) as a TLV.
    fn qc_oid(arcs: &[u8]) -> Vec<u8> {
        // 0.4 -> 0x04, 0 -> 0x00, 1862 -> 0x8e 0x46, then the given arcs.
        let mut content = vec![0x04, 0x00, 0x8e, 0x46, 0x01];
        content.extend_from_slice(arcs);
        tlv(0x06, &content)
    }

    fn statement(id: &[u8], info: Option<Vec<u8>>) -> Vec<u8> {
        let mut body = qc_oid(id);
        body.extend(info.unwrap_or_default());
        tlv(0x30, &body)
    }

    fn extension(statements: &[Vec<u8>]) -> Vec<u8> {
        tlv(0x30, &statements.concat())
    }

    #[test]
    fn empty_extension_is_all_false() {
        assert_eq!(
            Qualified::from_extension(&extension(&[])).unwrap(),
            Qualified::default()
        );
    }

    #[test]
    fn compliance_and_sscd_are_flags() {
        let ext = extension(&[statement(&[0x01], None), statement(&[0x04], None)]);
        let q = Qualified::from_extension(&ext).unwrap();
        assert!(q.compliance);
        assert!(q.sscd);
        assert!(q.types.is_empty());
    }

    #[test]
    fn types_keep_certificate_order_and_skip_unknown_oids() {
        let types = tlv(
            0x30,
            &[
                qc_oid(&[0x06, 0x03]),
                qc_oid(&[0x06, 0x09]),
                qc_oid(&[0x06, 0x01]),
                qc_oid(&[0x06, 0x02]),
            ]
            .concat(),
        );
        let ext = extension(&[statement(&[0x06], Some(types))]);
        let q = Qualified::from_extension(&ext).unwrap();
        assert_eq!(q.types, vec![QcType::Web, QcType::ESign, QcType::ESeal]);
    }

    #[test]
    fn unknown_statements_and_missing_type_info_are_ignored() {
        let ext = extension(&[
            statement(&[0x63], Some(tlv(0x05, &[]))),
            statement(&[0x06], None),
        ]);
        assert_eq!(
            Qualified::from_extension(&ext).unwrap(),
            Qualified::default()
        );
    }

    #[test]
    fn malformed_structures_are_errors() {
        // QcType info that is not a SEQUENCE.
        let ext = extension(&[statement(&[0x06], Some(tlv(0x04, &[1])))]);
        assert!(Qualified::from_extension(&ext).is_err());
        // Statement that is not a SEQUENCE starting with an OID.
        assert!(Qualified::from_extension(&extension(&[tlv(0x30, &tlv(0x02, &[1]))])).is_err());
        // Not a SEQUENCE OF.
        assert!(Qualified::from_extension(&tlv(0x04, &[])).is_err());
        assert!(Qualified::from_extension(&[]).is_err());
    }
}
