//! The window must be on screen within 300 ms of a request (`docs/plan.md`
//! B4). Measured headless: from the `Open` command to the frames with the
//! certificate list laid out and tessellated, the part this crate controls
//! (asserted), then rasterized (printed: on CI machines the rasterizer is
//! software, llvmpipe or WARP, and its speed is not ours to budget).

use std::time::{Duration, Instant};

use websign_protocol::types::BrowserName;

use super::fixtures::*;
use super::support::Rig;

const BUDGET: Duration = Duration::from_millis(300);

#[test]
fn the_first_frame_is_ready_within_300_ms() {
    let started = Instant::now();
    let mut rig = Rig::new(false);
    rig.harness.render().expect("a first frame");
    let renderer = started.elapsed();

    let opened = Instant::now();
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, true));
    rig.list(vec![ana_a3(), ana_a1(), clinic_a1(), old_a3()], Vec::new());
    let laid_out = opened.elapsed();
    rig.harness.render().expect("the list frame");
    let drawn = opened.elapsed();

    eprintln!(
        "confirmation window: frames laid out {} ms after Open, drawn at {} ms \
         (software renderer; its own start took {} ms)",
        laid_out.as_millis(),
        drawn.as_millis(),
        renderer.as_millis()
    );
    assert!(laid_out < BUDGET, "{laid_out:?} ≥ {BUDGET:?}");
}
