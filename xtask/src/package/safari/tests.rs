use super::*;
use crate::package::project;
use crate::testutil::TempDir;

fn context(out: &TempDir) -> Context {
    let root = crate::root::repo_root().unwrap();
    let (project, version) = project::load(&root).unwrap();
    Context {
        root,
        out: out.path().to_path_buf(),
        project,
        version,
        no_build: true,
    }
}

/// The keys `safari/Relay/BundledHost.swift` and Safari read.
#[test]
fn the_appex_info_plist_names_the_extension_point_and_the_host() {
    let out = TempDir::new();
    let context = context(&out);
    let plist = out.path().join("Info.plist");
    let executable = appex_executable(&context.project);
    render_template(
        &context,
        "macos/Extension-Info.plist.in",
        &plist,
        &[("appex_executable", &executable)],
    )
    .unwrap();
    let text = std::fs::read_to_string(plist).unwrap();
    let project = &context.project;
    // Apple requires an appex's ID to extend its containing app's.
    assert!(
        project
            .safari_bundle_id
            .starts_with(&format!("{}.", project.bundle_id))
    );
    for line in [
        "<string>com.apple.Safari.web-extension</string>".to_owned(),
        "<string>SafariWebExtensionHandler</string>".to_owned(),
        "<string>XPC!</string>".to_owned(),
        format!("<string>{}</string>", project.safari_bundle_id),
        format!("<string>{executable}</string>"),
        format!(
            "<key>SignLocalHostExecutable</key>\n\t<string>{}</string>",
            project.slug
        ),
        format!(
            "<key>SignLocalHostExtensionID</key>\n\t<string>{}</string>",
            project.safari_bundle_id
        ),
    ] {
        assert!(text.contains(&line), "missing {line} in\n{text}");
    }
    assert!(!text.contains("{{"));
}

#[test]
fn every_swift_file_of_the_relay_and_the_handler_is_compiled() {
    let root = crate::root::repo_root().unwrap();
    let sources = swift_sources(&root).unwrap();
    let names: Vec<String> = sources
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    for wanted in [
        "Relay.swift",
        "BundledHost.swift",
        "SafariWebExtensionHandler.swift",
    ] {
        assert!(names.iter().any(|name| name == wanted), "{names:?}");
    }
    assert!(names.iter().all(|name| name.ends_with(".swift")));
}

#[test]
fn swiftc_links_an_extension_entry_point_and_ends_with_the_sources() {
    let sources = [PathBuf::from("a.swift"), PathBuf::from("b.swift")];
    let args = swiftc_args("arm64-apple-macos13.0", Path::new("out"), &sources);
    let text: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert_eq!(text[0], "swiftc");
    let entry = text
        .iter()
        .position(|arg| arg == "_NSExtensionMain")
        .unwrap();
    assert_eq!(text[entry - 2..entry], ["-e", "-Xlinker"]);
    assert!(text.contains(&"-parse-as-library".to_owned()));
    assert_eq!(text[text.len() - 3..], ["out", "a.swift", "b.swift"]);
}

#[test]
fn a_missing_web_build_says_how_to_make_it() {
    let empty = TempDir::new();
    let error = web_build(empty.path()).unwrap_err();
    assert!(error.contains("wxt build -b safari"), "{error}");
}

/// Apple: a helper started by a sandboxed process carries exactly these two
/// keys, or it crashes at launch.
#[test]
fn the_host_copy_only_inherits_the_sandbox() {
    let root = crate::root::repo_root().unwrap();
    let text =
        std::fs::read_to_string(root.join("packaging/macos/safari-host.entitlements")).unwrap();
    let keys: Vec<&str> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("<key>")?.strip_suffix("</key>"))
        .collect();
    assert_eq!(
        keys,
        [
            "com.apple.security.app-sandbox",
            "com.apple.security.inherit"
        ]
    );
    let appex = std::fs::read_to_string(root.join("packaging/macos/safari-extension.entitlements"))
        .unwrap();
    assert!(appex.contains("<key>com.apple.security.app-sandbox</key>\n\t<true/>"));
}

#[test]
fn folders_are_copied_recursively() {
    let from = TempDir::new();
    from.write("manifest.json", "{}");
    from.write("assets/icon.png", "png");
    let to = TempDir::new();
    copy_tree(from.path(), &to.path().join("Resources")).unwrap();
    assert!(to.path().join("Resources/manifest.json").is_file());
    assert!(to.path().join("Resources/assets/icon.png").is_file());
}
