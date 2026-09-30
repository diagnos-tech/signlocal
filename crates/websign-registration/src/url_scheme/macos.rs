//! macOS: the scheme is declared by the `.app` bundle, so there is nothing to
//! write at run time; packaging takes the declaration from here.

use websign_project::{MACOS_BUNDLE_ID, URL_SCHEME};

use crate::destination::Outcome;

pub fn skipped() -> Outcome {
    Outcome::Skipped("declared in Info.plist".into())
}

/// The `CFBundleURLTypes` entry for `Info.plist` (a `<key>` and its
/// `<array>`), so the packaging and this crate cannot disagree on the scheme.
pub fn info_plist_url_types() -> String {
    format!(
        "<key>CFBundleURLTypes</key>\n\
         <array>\n\
         \t<dict>\n\
         \t\t<key>CFBundleURLName</key>\n\
         \t\t<string>{MACOS_BUNDLE_ID}</string>\n\
         \t\t<key>CFBundleURLSchemes</key>\n\
         \t\t<array>\n\
         \t\t\t<string>{URL_SCHEME}</string>\n\
         \t\t</array>\n\
         \t</dict>\n\
         </array>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plist_fragment_declares_exactly_our_scheme() {
        let fragment = info_plist_url_types();
        assert!(fragment.starts_with("<key>CFBundleURLTypes</key>"));
        assert!(fragment.contains(&format!("<string>{URL_SCHEME}</string>")));
        assert_eq!(fragment.matches("<dict>").count(), 1);
    }
}
