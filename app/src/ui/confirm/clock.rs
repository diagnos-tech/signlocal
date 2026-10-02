//! Where the window reads the time.
//!
//! The model takes every instant as an argument (`websign-ui-model`), so the
//! window only has to pick the source: the system clock, or a hand-moved one
//! that lets tests cross the 600 ms arming and the 150 ms skeleton delay
//! without sleeping.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// A source of instants.
#[derive(Debug, Clone)]
pub enum Clock {
    System,
    /// Tests: moves only when told to.
    Manual(Rc<Cell<Instant>>),
}

impl Clock {
    /// A manual clock starting now, and the handle that moves it.
    pub fn manual() -> (Clock, ManualTime) {
        let cell = Rc::new(Cell::new(Instant::now()));
        (Clock::Manual(Rc::clone(&cell)), ManualTime(cell))
    }

    pub fn now(&self) -> Instant {
        match self {
            Clock::System => Instant::now(),
            Clock::Manual(cell) => cell.get(),
        }
    }
}

/// The hand of a [`Clock::manual`].
#[derive(Debug, Clone)]
pub struct ManualTime(Rc<Cell<Instant>>);

impl ManualTime {
    pub fn advance(&self, by: Duration) {
        self.0.set(self.0.get() + by);
    }
}
