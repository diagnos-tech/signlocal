//! One snapshot per state and theme, `<scene>-<theme>.png`.

use super::scenes::SCENES;
use super::support::{Rig, THEMES};

#[test]
fn every_state_in_both_themes() {
    for (dark, theme) in THEMES {
        for (name, scene) in SCENES {
            let mut rig = Rig::new(dark);
            scene(&mut rig);
            rig.snapshot(&format!("{name}-{theme}"));
        }
    }
}
