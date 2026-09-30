//! Distinguished-name extraction.

use super::DistinguishedName;
use super::der::{self, DerError, OBJECT_IDENTIFIER, Reader, SEQUENCE, SET};
use super::oid::{COMMON_NAME, COUNTRY_NAME, ORGANIZATION_NAME, ORGANIZATIONAL_UNIT_NAME};
use super::strings;

impl DistinguishedName {
    /// Reads CN, O, OU and C from the contents of a `Name`.
    ///
    /// Attributes whose string type is not supported, or whose bytes are not
    /// valid for it, are skipped as if absent, so a single-valued attribute
    /// that appears more than once keeps its first *readable* value. The
    /// structure itself (SETs of type-value pairs) must be well formed.
    pub(super) fn from_name(name: &[u8]) -> Result<Self, DerError> {
        let mut dn = Self::default();
        for rdn in der::elements(name, SET) {
            for attribute in der::elements(rdn?, SEQUENCE) {
                let mut fields = Reader::new(attribute?);
                let kind = fields.read(OBJECT_IDENTIFIER)?;
                let value = fields.read_any()?;
                fields.finish()?;
                let Some(text) = strings::decode(value.tag, value.content) else {
                    continue;
                };
                match kind {
                    COMMON_NAME => _ = dn.common_name.get_or_insert(text),
                    ORGANIZATION_NAME => _ = dn.organization.get_or_insert(text),
                    ORGANIZATIONAL_UNIT_NAME => dn.organizational_units.push(text),
                    COUNTRY_NAME => _ = dn.country.get_or_insert(text),
                    _ => {}
                }
            }
        }
        Ok(dn)
    }
}
