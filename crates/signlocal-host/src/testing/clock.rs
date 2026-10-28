//! A clock that only moves when told to.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime};

use crate::ports::Clock;

/// Shared by clones: the engine holds one, the test another.
#[derive(Debug, Clone)]
pub struct ManualClock {
    now: Rc<Cell<Instant>>,
    wall: Rc<Cell<SystemTime>>,
}

impl Default for ManualClock {
    fn default() -> Self {
        ManualClock {
            now: Rc::new(Cell::new(Instant::now())),
            wall: Rc::new(Cell::new(SystemTime::now())),
        }
    }
}

impl ManualClock {
    /// Moves both clocks forward.
    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get() + by);
        self.wall.set(self.wall.get() + by);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn wall(&self) -> SystemTime {
        self.wall.get()
    }
}
