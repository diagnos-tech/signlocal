//! The balance logic behind [`super::SecureInput`], generic over the guard
//! so tests can count enables and disables with a fake.

/// Holds at most one guard: one is acquired when the wanted state turns on
/// and dropped when it turns off or the latch is dropped (also while
/// unwinding from a panic), so acquisitions and releases always pair up.
#[derive(Debug)]
pub struct Latch<G> {
    wanted: bool,
    guard: Option<G>,
}

impl<G> Default for Latch<G> {
    fn default() -> Self {
        Self {
            wanted: false,
            guard: None,
        }
    }
}

impl<G> Latch<G> {
    /// Moves to `on`. Only an off-to-on change calls `acquire`; when it
    /// fails (`None`) the latch stays off until the next off-to-on change,
    /// so a refusing OS is not asked again every frame.
    pub fn set(&mut self, on: bool, acquire: impl FnOnce() -> Option<G>) {
        if on == self.wanted {
            return;
        }
        self.wanted = on;
        self.guard = if on { acquire() } else { None };
    }

    /// Whether a guard is held.
    pub fn is_held(&self) -> bool {
        self.guard.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI32, Ordering};

    use super::Latch;

    /// Counts live guards: +1 when made, -1 when dropped.
    #[derive(Debug)]
    struct Fake(Arc<AtomicI32>);

    impl Drop for Fake {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn acquire(count: &Arc<AtomicI32>) -> impl FnOnce() -> Option<Fake> + '_ {
        || {
            count.fetch_add(1, Ordering::SeqCst);
            Some(Fake(Arc::clone(count)))
        }
    }

    #[test]
    fn repeated_on_enables_once_and_off_disables_once() {
        let count = Arc::new(AtomicI32::new(0));
        let mut latch = Latch::default();
        for _ in 0..3 {
            latch.set(true, acquire(&count));
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(latch.is_held());
        for _ in 0..3 {
            latch.set(false, acquire(&count));
        }
        assert_eq!(count.load(Ordering::SeqCst), 0);
        assert!(!latch.is_held());
    }

    #[test]
    fn dropping_while_on_disables() {
        let count = Arc::new(AtomicI32::new(0));
        let mut latch = Latch::default();
        latch.set(true, acquire(&count));
        drop(latch);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_panic_while_on_disables_during_unwinding() {
        let count = Arc::new(AtomicI32::new(0));
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut latch = Latch::default();
            latch.set(true, acquire(&count));
            assert_eq!(count.load(Ordering::SeqCst), 1);
            panic!("the window crashed with the PIN field focused");
        }));
        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_refused_enable_is_not_retried_until_the_next_focus() {
        let calls = AtomicI32::new(0);
        let refuse = || {
            calls.fetch_add(1, Ordering::SeqCst);
            None::<Fake>
        };
        let mut latch = Latch::default();
        latch.set(true, refuse);
        latch.set(true, refuse);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!latch.is_held());

        let count = Arc::new(AtomicI32::new(0));
        latch.set(false, acquire(&count));
        latch.set(true, acquire(&count));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(latch.is_held());
    }
}
