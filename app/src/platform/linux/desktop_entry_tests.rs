use super::*;

fn entry(name: &str, program: &str) -> Option<Entry> {
    Some(Entry {
        name: name.into(),
        program: program.into(),
    })
}

#[test]
fn reads_name_and_program_from_the_main_group() {
    let text = "# comment\n[Desktop Entry]\nType=Application\nName=Firefox\n\
                Name[pt_BR]=Navegador\nExec=/usr/lib/firefox/firefox %u\n\
                [Desktop Action new-window]\nName=New Window\nExec=other\n";
    assert_eq!(parse(text), entry("Firefox", "/usr/lib/firefox/firefox"));
}

#[test]
fn skips_links_hidden_and_incomplete_entries() {
    assert_eq!(parse("[Desktop Entry]\nType=Link\nName=A\nExec=a\n"), None);
    let hidden = "[Desktop Entry]\nType=Application\nName=A\nExec=a\nHidden=true\n";
    assert_eq!(parse(hidden), None);
    assert_eq!(parse("[Desktop Entry]\nType=Application\nName=A\n"), None);
    assert_eq!(
        parse("[Desktop Entry]\nType=Application\nName=\nExec=a\n"),
        None
    );
    assert_eq!(parse("[Other]\nType=Application\nName=A\nExec=a\n"), None);
}

#[test]
fn unquotes_and_skips_env_prefixes() {
    assert_eq!(
        first_program(r#""/opt/My App/run\"x\"" --flag"#).as_deref(),
        Some(r#"/opt/My App/run"x""#)
    );
    assert_eq!(
        first_program("env GDK_BACKEND=x11 LANG=C signer %f").as_deref(),
        Some("signer")
    );
    assert_eq!(first_program("   ").as_deref(), None);
    assert_eq!(first_program("env A=1").as_deref(), None);
}

#[test]
fn resolves_through_path_and_symlinks() {
    let dir = scratch("resolve");
    let real = dir.join("real-program");
    std::fs::write(&real, "#!/bin/sh\n").unwrap();
    std::os::unix::fs::symlink(&real, dir.join("alias")).unwrap();
    let search = OsString::from(format!("/nonexistent:{}", dir.display()));
    let real = std::fs::canonicalize(&real).unwrap();
    assert_eq!(resolve("alias", &search), Some(real.clone()));
    assert_eq!(resolve(real.to_str().unwrap(), &search), Some(real));
    assert_eq!(resolve("sub/alias", &search), None);
    assert_eq!(resolve("missing", &search), None);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn finds_the_entry_that_starts_the_executable() {
    let dir = scratch("entries");
    let program = dir.join("signer-app");
    std::fs::write(&program, "").unwrap();
    let program = std::fs::canonicalize(program).unwrap();
    std::fs::write(dir.join("a-other.desktop"), launcher("Other", "/bin/sh")).unwrap();
    std::fs::write(
        dir.join("b-signer.desktop"),
        launcher("Signer", program.to_str().unwrap()),
    )
    .unwrap();
    std::fs::write(
        dir.join("c-notes.txt"),
        launcher("Wrong", program.to_str().unwrap()),
    )
    .unwrap();
    let search = OsString::new();
    assert_eq!(name_in(&dir, &program, &search).as_deref(), Some("Signer"));
    assert_eq!(name_in(&dir, Path::new("/nowhere"), &search), None);
    std::fs::remove_dir_all(dir).unwrap();
}

fn launcher(name: &str, exec: &str) -> String {
    format!("[Desktop Entry]\nType=Application\nName={name}\nExec=\"{exec}\" %U\n")
}

fn scratch(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "websign-desktop-entry-{}-{test}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
