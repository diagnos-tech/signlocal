//! The two waits the client bounds itself; every other wait is bounded by
//! the app (the person's decision time) or by the pipe closing.

use std::time::Duration;

/// How long to wait for the app where a hang would otherwise be possible.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Timing {
    /// For the answer to `hello`. Longer than the app's own five-second
    /// start-up budget so a slow disk reads as "not responding" here rather
    /// than as a race.
    pub hello_reply: Duration,
    /// For the app to exit after its stdin closes, and for its final answer
    /// after a `cancel`. A healthy app is far quicker; the bound exists so a
    /// wedged one cannot block `Drop` or a failed `sign`.
    pub exit_grace: Duration,
}

impl Timing {
    pub(crate) const DEFAULT: Timing = Timing {
        hello_reply: Duration::from_secs(10),
        exit_grace: Duration::from_secs(5),
    };
}
