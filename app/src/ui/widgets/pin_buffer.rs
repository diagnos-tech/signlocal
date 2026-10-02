//! Editing a PIN without leaving copies of it in memory.
//!
//! A `String` that grows reallocates and frees its old buffer unwiped, so
//! growth here is manual: copy into a larger buffer, then zeroize the old
//! one. `Zeroizing` wipes the whole allocation (spare capacity included)
//! when the caller drops or clears it after `C_Login`.

use zeroize::{Zeroize, Zeroizing};

/// Room for a 16-character PIN in any script without growing.
const INITIAL_CAPACITY: usize = 64;

/// Appends the printable characters of `typed` while the PIN has fewer than
/// `max_chars` characters; returns whether anything was added.
pub fn push(pin: &mut Zeroizing<String>, typed: &str, max_chars: usize) -> bool {
    let mut changed = false;
    for ch in typed.chars().filter(|ch| !ch.is_control()) {
        if pin.chars().count() >= max_chars {
            break;
        }
        reserve(pin, ch.len_utf8());
        pin.push(ch);
        changed = true;
    }
    changed
}

/// Removes the last character; returns whether there was one.
pub fn pop(pin: &mut Zeroizing<String>) -> bool {
    pin.pop().is_some()
}

/// Makes room for `extra` bytes without an unwiped reallocation.
fn reserve(pin: &mut Zeroizing<String>, extra: usize) {
    let needed = pin.len() + extra;
    if needed <= pin.capacity() {
        return;
    }
    let mut grown = String::with_capacity(needed.max(2 * pin.capacity()).max(INITIAL_CAPACITY));
    grown.push_str(pin.as_str());
    pin.zeroize();
    **pin = grown;
}

/// What the field shows while hidden: one bullet per character, also the
/// only value AccessKit ever gets.
pub fn mask(pin: &str) -> String {
    "•".repeat(pin.chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_printable_characters_up_to_the_limit() {
        let mut pin = Zeroizing::new(String::new());
        assert!(push(&mut pin, "12\n3", 16));
        assert_eq!(pin.as_str(), "123");
        assert!(push(&mut pin, "4567", 5));
        assert_eq!(pin.as_str(), "12345");
        assert!(!push(&mut pin, "6", 5));
    }

    #[test]
    fn grows_once_and_keeps_room() {
        let mut pin = Zeroizing::new(String::new());
        push(&mut pin, "1", 16);
        let capacity = pin.capacity();
        assert!(capacity >= INITIAL_CAPACITY);
        push(&mut pin, "234567890abcdef", 16);
        assert_eq!(
            pin.capacity(),
            capacity,
            "a 16-char PIN must not reallocate"
        );
    }

    #[test]
    fn pops_and_masks() {
        let mut pin = Zeroizing::new("ab".to_owned());
        assert_eq!(mask(&pin), "••");
        assert!(pop(&mut pin));
        assert!(pop(&mut pin));
        assert!(!pop(&mut pin));
        assert_eq!(mask(&pin), "");
    }
}
