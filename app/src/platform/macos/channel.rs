//! Mac App Store or not: store builds run in the App Sandbox, direct
//! builds do not (`docs/plan.md` D10).

use std::ffi::c_void;

use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::string::{CFString, CFStringRef};

use crate::platform::channel::InstallFormat;

// Not wrapped by `security-framework`.
#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecTaskCreateFromSelf(allocator: *const c_void) -> CFTypeRef;
    fn SecTaskCopyValueForEntitlement(
        task: CFTypeRef,
        entitlement: CFStringRef,
        error: *mut CFTypeRef,
    ) -> CFTypeRef;
}

pub fn install_format() -> InstallFormat {
    if entitled("com.apple.security.app-sandbox") {
        InstallFormat::MacAppStore
    } else {
        InstallFormat::Archive
    }
}

/// Whether our own code signature grants `entitlement` as `true`. Read
/// from the kernel's view of this process, not from an environment
/// variable a parent could set.
fn entitled(entitlement: &str) -> bool {
    // SAFETY: the default allocator (null); a +1 task or null.
    let task = unsafe { SecTaskCreateFromSelf(std::ptr::null()) };
    if task.is_null() {
        return false;
    }
    // SAFETY: a +1 object from a `Create` call, released on drop.
    let task = unsafe { CFType::wrap_under_create_rule(task) };
    let key = CFString::new(entitlement);
    // SAFETY: live task and key; the error out-pointer may be null.
    let value = unsafe {
        SecTaskCopyValueForEntitlement(
            task.as_CFTypeRef(),
            key.as_concrete_TypeRef(),
            std::ptr::null_mut(),
        )
    };
    if value.is_null() {
        return false;
    }
    // SAFETY: a +1 object from a `Copy` call, released on drop.
    let value = unsafe { CFType::wrap_under_create_rule(value) };
    value.downcast::<CFBoolean>().is_some_and(bool::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runs_are_not_sandboxed() {
        assert_eq!(install_format(), InstallFormat::Archive);
    }
}
