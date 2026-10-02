//! Which subject attribute names the holder (`docs/ux.md` §5.2).

use super::CertInfo;

impl CertInfo {
    /// The first non-blank of: the ICP-Brasil holder (CN without its document
    /// suffix), the CN, `givenName` + `surname` (both required: eIDAS
    /// personal certificates often have no CN), and the organization.
    ///
    /// A person's name comes before the organization because a personal
    /// certificate's `O` is usually the employer, not the holder.
    pub(crate) fn name_candidate(&self) -> Option<String> {
        let subject = &self.subject;
        let holder = self
            .icp_brasil
            .as_ref()
            .and_then(|icp| icp.holder_name.as_deref());
        let given_and_surname = || {
            let given = non_blank(subject.given_name.as_deref())?;
            let surname = non_blank(subject.surname.as_deref())?;
            Some(format!("{} {}", given.trim(), surname.trim()))
        };
        non_blank(holder)
            .or_else(|| non_blank(subject.common_name.as_deref()))
            .map(str::to_owned)
            .or_else(given_and_surname)
            .or_else(|| non_blank(subject.organization.as_deref()).map(str::to_owned))
    }
}

fn non_blank(text: Option<&str>) -> Option<&str> {
    text.filter(|text| !text.trim().is_empty())
}
