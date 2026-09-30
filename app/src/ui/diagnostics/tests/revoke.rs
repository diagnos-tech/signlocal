//! Revoking a remembered site: "Revoke" turns into "Confirm revoke", the
//! second press removes the site from the consent store on disk, and the
//! window says so.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_host::store::{DiskStores, Stores};
use websign_protocol::messages::DiagnosticsTab;

use super::support::{Setup, open};

/// A folder of its own under the system temp dir, removed when dropped.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!("websign-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn revoke_asks_once_more_then_forgets_the_site() {
    let dir = TempDir::new("revoke");
    let mut seed = DiskStores::new(&dir.0);
    let consent = seed.consent();
    consent
        .remember("https://app.diagnos.health", "aa", 1_789_000_000)
        .unwrap_or_else(|error| panic!("{error}"));
    consent
        .remember("https://laudos.clinica.med.br", "bb", 1_789_000_000)
        .unwrap_or_else(|error| panic!("{error}"));

    let mut window = open(Setup {
        stores: Box::new(DiskStores::new(&dir.0)),
        tab: Some(DiagnosticsTab::Browsers),
        height: 1000.0,
        ..Setup::default()
    });
    let revokes: Vec<_> = window
        .harness
        .get_all_by_role_and_label(Role::Button, "Revoke")
        .collect();
    assert_eq!(revokes.len(), 2);
    revokes[0].click();
    drop(revokes);
    window.harness.run();
    let confirm = window
        .harness
        .get_by_role_and_label(Role::Button, "Confirm revoke");
    confirm.click();
    window.harness.run();

    let left = DiskStores::new(&dir.0)
        .consent()
        .list()
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(left.len(), 1, "exactly one site was revoked");
    assert_eq!(
        window
            .harness
            .get_all_by_role_and_label(Role::Button, "Revoke")
            .count(),
        1
    );
    window
        .harness
        .get_by_label_contains("is no longer allowed.");
    assert!(
        window
            .harness
            .query_by_role_and_label(Role::Button, "Confirm revoke")
            .is_none()
    );
}

#[test]
fn an_unconfirmed_revoke_expires() {
    let dir = TempDir::new("revoke-expires");
    DiskStores::new(&dir.0)
        .consent()
        .remember("https://app.diagnos.health", "aa", 1_789_000_000)
        .unwrap_or_else(|error| panic!("{error}"));
    let mut window = open(Setup {
        stores: Box::new(DiskStores::new(&dir.0)),
        tab: Some(DiagnosticsTab::Browsers),
        height: 1000.0,
        ..Setup::default()
    });
    window
        .harness
        .get_by_role_and_label(Role::Button, "Revoke")
        .click();
    window.harness.run();
    window
        .harness
        .get_by_role_and_label(Role::Button, "Confirm revoke");
    // Past the 4 s confirmation window.
    window.window().state.revoke_armed = Some(("https://app.diagnos.health".to_owned(), -1.0));
    window.harness.run();
    window.harness.get_by_role_and_label(Role::Button, "Revoke");
    let left = DiskStores::new(&dir.0).consent().list().unwrap_or_default();
    assert_eq!(left.len(), 1, "nothing was revoked");
}
