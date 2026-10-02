//! The product name of a macOS program: its enclosing `.app`'s
//! `CFBundleDisplayName` or `CFBundleName`.

use std::path::Path;

use core_foundation::bundle::CFBundle;
use core_foundation::string::CFString;
use core_foundation::url::CFURL;

pub fn product_name(executable: &Path) -> Option<String> {
    let app = enclosing_app(executable)?;
    let bundle = CFBundle::new(CFURL::from_path(app, true)?)?;
    let info = bundle.info_dictionary();
    ["CFBundleDisplayName", "CFBundleName"]
        .iter()
        .find_map(|key| {
            let value = info.find(CFString::new(key))?;
            let name = value.downcast::<CFString>()?.to_string();
            (!name.trim().is_empty()).then_some(name)
        })
}

/// The innermost `.app` folder holding `executable` (a helper app inside
/// another app names itself, not its host). Plain tools have none.
fn enclosing_app(executable: &Path) -> Option<&Path> {
    executable
        .ancestors()
        .skip(1)
        .find(|dir| dir.extension().is_some_and(|ext| ext == "app"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_innermost_app() {
        let helper = Path::new("/Applications/A.app/Contents/Frameworks/H.app/Contents/MacOS/h");
        assert_eq!(
            enclosing_app(helper),
            Some(Path::new("/Applications/A.app/Contents/Frameworks/H.app"))
        );
        assert_eq!(enclosing_app(Path::new("/usr/bin/python3")), None);
    }

    #[test]
    fn reads_a_system_app_name() {
        let finder = Path::new("/System/Library/CoreServices/Finder.app/Contents/MacOS/Finder");
        assert_eq!(product_name(finder).as_deref(), Some("Finder"));
    }
}
