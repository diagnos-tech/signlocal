//! Reading certificates and private key identifiers from a token session.

use cryptoki::error::Result;
use cryptoki::object::{Attribute, AttributeType, CertificateType, ObjectClass};
use cryptoki::session::Session;

/// An X.509 certificate object on a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCertificate {
    pub der: Vec<u8>,
    /// `CKA_ID`: the link to the private key. Empty when the token sets none.
    pub id: Vec<u8>,
}

/// Every X.509 certificate the session can see. A certificate object that
/// cannot be read is skipped so it does not hide the others.
pub fn certificates(session: &Session) -> Result<Vec<StoredCertificate>> {
    let handles = session.find_objects(&[Attribute::Class(ObjectClass::CERTIFICATE)])?;
    let wanted = [
        AttributeType::Value,
        AttributeType::Id,
        AttributeType::CertificateType,
    ];
    let mut found = Vec::new();
    for handle in handles {
        let Ok(attributes) = session.get_attributes(handle, &wanted) else {
            continue;
        };
        let (mut der, mut id, mut x509) = (Vec::new(), Vec::new(), true);
        for attribute in attributes {
            match attribute {
                Attribute::Value(value) => der = value,
                Attribute::Id(value) => id = value,
                Attribute::CertificateType(kind) => x509 = kind == CertificateType::X_509,
                _ => {}
            }
        }
        if x509 && !der.is_empty() {
            found.push(StoredCertificate { der, id });
        }
    }
    Ok(found)
}

/// `CKA_ID` of every private key visible without logging in.
///
/// Errors count as "none visible": tokens that hide private objects until
/// the PIN is given may refuse the search outright.
pub fn visible_private_key_ids(session: &Session) -> Vec<Vec<u8>> {
    let Ok(handles) = session.find_objects(&[Attribute::Class(ObjectClass::PRIVATE_KEY)]) else {
        return Vec::new();
    };
    handles
        .into_iter()
        .filter_map(|handle| session.get_attributes(handle, &[AttributeType::Id]).ok())
        .filter_map(|attributes| {
            attributes
                .into_iter()
                .find_map(|attribute| match attribute {
                    Attribute::Id(id) => Some(id),
                    _ => None,
                })
        })
        .collect()
}
