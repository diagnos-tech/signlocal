//! Distinguished-name extraction.

use const_oid::db::rfc4519::{
    COMMON_NAME, COUNTRY_NAME, ORGANIZATION_NAME, ORGANIZATIONAL_UNIT_NAME,
};
use der::Tagged;
use x509_cert::name::Name;

use super::DistinguishedName;
use super::strings;

impl DistinguishedName {
    /// Reads CN, O, OU and C from `name`.
    ///
    /// A single-valued attribute that appears more than once keeps its first
    /// readable value; attributes whose string type is not supported are
    /// skipped as if absent.
    ///
    /// SPEC: "the first one wins" is read as the first *readable* one, so an
    /// unsupported first occurrence does not hide a later supported one.
    pub(super) fn from_name(name: &Name) -> Self {
        let mut dn = Self::default();
        for attribute in name.iter() {
            let value = &attribute.value;
            let Some(text) = strings::decode(value.tag(), value.value()) else {
                continue;
            };
            let oid = attribute.oid;
            if oid == COMMON_NAME {
                dn.common_name.get_or_insert(text);
            } else if oid == ORGANIZATION_NAME {
                dn.organization.get_or_insert(text);
            } else if oid == ORGANIZATIONAL_UNIT_NAME {
                dn.organizational_units.push(text);
            } else if oid == COUNTRY_NAME {
                dn.country.get_or_insert(text);
            }
        }
        dn
    }
}
