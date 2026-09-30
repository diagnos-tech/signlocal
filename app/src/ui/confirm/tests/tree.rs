//! The whole AccessKit tree of every state (`docs/ux.md` §14): every node a
//! screen reader reaches is named in words and placed on screen, in both
//! themes' scenes (the tree does not depend on the theme, so light only).

use egui_kittest::kittest::Queryable as _;

use super::scenes::SCENES;
use super::support::Rig;
use crate::ui::audit;

#[test]
fn every_state_reads_as_words() {
    for (name, scene) in SCENES {
        let mut rig = Rig::new(false);
        scene(&mut rig);
        let problems = audit::problems(rig.harness.query_all_by(|_| true));
        assert!(problems.is_empty(), "{name}: {problems:#?}");
    }
}
