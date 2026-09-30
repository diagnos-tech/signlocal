//! The holder name a person recognizes (`docs/ux.md` §5.2, vectors §16.3).

use crate::cert::CertInfo;

/// The name to show for `info`: [`CertInfo::display_name`] (ICP-Brasil holder
/// without the document suffix, CN, O, or a fingerprint prefix), then, when
/// it has letters and all of them are uppercase, [`title_case`].
pub fn display_name(info: &CertInfo) -> String {
    let _ = info;
    todo!("SPEC.md §10")
}

/// `"JOAO DA SILVA DOS SANTOS"` → `"Joao da Silva dos Santos"`.
///
/// Each word gets an uppercase first letter and lowercase rest, except the
/// particles `da das de di do dos du e del la van von y` (lowercase unless
/// first) and the company suffixes `ME EPP EIRELI S.A. S/A` (kept as written;
/// `LTDA` becomes `Ltda`). Word separators are kept exactly.
pub fn title_case(name: &str) -> String {
    let _ = name;
    todo!("SPEC.md §10")
}
