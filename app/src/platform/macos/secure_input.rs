//! Carbon's secure keyboard entry, held by an RAII guard.
//!
//! macOS counts `EnableSecureEventInput` calls per process; while the count
//! is above zero, no other process can read keystrokes through event taps,
//! system-wide. An unbalanced call therefore breaks other apps' hotkeys and
//! input methods until we exit, so every enable is paired with exactly one
//! disable, in `Drop`.

// Not wrapped by any crate we use. Both return an `OSStatus`.
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn EnableSecureEventInput() -> i32;
    fn DisableSecureEventInput() -> i32;
}

/// Secure keyboard entry, on while this value lives.
#[derive(Debug)]
pub struct Guard {
    /// Only [`Guard::enable`] makes one, so each value owns one enable.
    _enabled: (),
}

impl Guard {
    /// Turns secure keyboard entry on, or `None` when macOS refused (then
    /// there is nothing to disable).
    pub fn enable() -> Option<Self> {
        // SAFETY: takes no arguments and only increments this process's
        // secure-input count.
        let status = unsafe { EnableSecureEventInput() };
        if status != 0 {
            log::warn!("secure keyboard entry did not turn on (OSStatus {status})");
            return None;
        }
        Some(Self { _enabled: () })
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        // SAFETY: takes no arguments; balances the successful enable this
        // value owns, so the count never goes below what others hold.
        let status = unsafe { DisableSecureEventInput() };
        if status != 0 {
            log::warn!("secure keyboard entry did not turn off (OSStatus {status})");
        }
    }
}
