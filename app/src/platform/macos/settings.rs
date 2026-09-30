//! Reduce motion and dark mode on macOS.

use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::string::{CFString, CFStringRef};

use super::objc::{AutoreleasePool, Id, ObjcBool, class, send0};

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFPreferencesCopyAppValue(key: CFStringRef, application: CFStringRef) -> CFTypeRef;
    static kCFPreferencesAnyApplication: CFStringRef;
}

/// System Settings › Accessibility › Display › "Reduce motion". Safe off
/// the main thread: `NSWorkspace` is thread-safe.
pub fn reduce_motion() -> bool {
    let Some(workspace) = class(c"NSWorkspace") else {
        return false;
    };
    let _pool = AutoreleasePool::new();
    // SAFETY: class and instance methods that exist with these signatures
    // (`+sharedWorkspace` → object, `-accessibility…` → BOOL).
    unsafe {
        let shared: Id = send0(workspace, c"sharedWorkspace");
        !shared.is_null()
            && send0::<ObjcBool>(shared, c"accessibilityDisplayShouldReduceMotion") != 0
    }
}

/// `AppleInterfaceStyle` is `Dark` in dark mode (also while "Auto" is dark)
/// and absent in light mode.
pub fn dark_mode() -> Option<bool> {
    let key = CFString::from_static_string("AppleInterfaceStyle");
    // SAFETY: a live key and the framework's constant; a +1 value or null.
    let value = unsafe {
        CFPreferencesCopyAppValue(key.as_concrete_TypeRef(), kCFPreferencesAnyApplication)
    };
    if value.is_null() {
        return Some(false);
    }
    // SAFETY: a +1 object from a `Copy` call, released on drop.
    let value = unsafe { CFType::wrap_under_create_rule(value) };
    Some(
        value
            .downcast::<CFString>()
            .is_some_and(|style| style == "Dark"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_answer_without_failing() {
        let _ = reduce_motion();
        assert!(dark_mode().is_some());
    }
}
