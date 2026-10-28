use serde::Deserialize;

use super::*;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Doc {
    version: u32,
    items: Vec<String>,
}

fn file(dir: &tempfile::TempDir) -> JsonFile {
    JsonFile {
        path: dir.path().join("doc.json"),
    }
}

#[test]
fn a_missing_file_reads_as_default() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(file(&dir).read::<Doc>().unwrap(), Doc::default());
}

#[test]
fn update_persists_and_creates_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonFile {
        path: dir.path().join("nested").join("doc.json"),
    };
    store
        .update(|doc: &mut Doc| {
            doc.version = 1;
            doc.items.push("a".into());
        })
        .unwrap();
    store
        .update(|doc: &mut Doc| doc.items.push("b".into()))
        .unwrap();
    let doc: Doc = store.read().unwrap();
    assert_eq!(doc.items, ["a", "b"]);
    assert!(!store.sibling("tmp").exists());
}

#[test]
fn a_corrupt_file_is_renamed_and_reads_as_default() {
    let dir = tempfile::tempdir().unwrap();
    let store = file(&dir);
    fs::write(&store.path, b"{not json").unwrap();
    assert_eq!(store.read::<Doc>().unwrap(), Doc::default());
    assert!(!store.path.exists());
    assert_eq!(fs::read(store.sibling("corrupt")).unwrap(), b"{not json");

    fs::write(&store.path, b"\"text\"").unwrap();
    assert_eq!(store.read::<Doc>().unwrap(), Doc::default());
    assert_eq!(fs::read(store.sibling("corrupt")).unwrap(), b"\"text\"");
}

#[test]
fn a_held_lock_times_out_with_an_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = file(&dir);
    let held = store.lock().unwrap();
    let started = Instant::now();
    let error = store.update(|_: &mut Doc| ()).unwrap_err();
    assert!(matches!(error, StoreError::Io { .. }));
    assert!(started.elapsed() >= LOCK_WAIT);
    drop(held);
    assert!(store.update(|_: &mut Doc| ()).is_ok());
}
