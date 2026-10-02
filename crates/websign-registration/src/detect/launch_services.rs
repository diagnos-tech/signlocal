//! LaunchServices lookups by bundle ID, and the bundle's version, through
//! the C APIs of CoreServices and CoreFoundation.

use std::ffi::{CStr, c_char, c_void};

type CFTypeRef = *const c_void;

const UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithBytes(
        allocator: CFTypeRef,
        bytes: *const u8,
        length: isize,
        encoding: u32,
        is_external: u8,
    ) -> CFTypeRef;
    fn CFRelease(value: CFTypeRef);
    fn CFArrayGetCount(array: CFTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFTypeRef, index: isize) -> CFTypeRef;
    fn CFBundleCreate(allocator: CFTypeRef, url: CFTypeRef) -> CFTypeRef;
    fn CFBundleGetValueForInfoDictionaryKey(bundle: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
    fn CFGetTypeID(value: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringGetCString(string: CFTypeRef, buffer: *mut c_char, size: isize, encoding: u32)
    -> u8;
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSCopyApplicationURLsForBundleIdentifier(
        bundle_id: CFTypeRef,
        error: *mut CFTypeRef,
    ) -> CFTypeRef;
}

/// A CoreFoundation object this code owns (from a Create/Copy call).
struct Owned(CFTypeRef);

impl Owned {
    /// `None` for null. Lazy on purpose: an eagerly built `Self(null)` would
    /// be dropped at once, and `CFRelease(NULL)` traps (SIGTRAP).
    fn new(value: CFTypeRef) -> Option<Self> {
        (!value.is_null()).then(|| Self(value))
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: `Owned` only wraps non-null objects from Create/Copy calls,
        // released exactly once here.
        unsafe { CFRelease(self.0) }
    }
}

fn cf_string(text: &str) -> Option<Owned> {
    let length = isize::try_from(text.len()).ok()?;
    // SAFETY: `text` is valid for `length` bytes of UTF-8 and is copied.
    Owned::new(unsafe { CFStringCreateWithBytes(std::ptr::null(), text.as_ptr(), length, UTF8, 0) })
}

/// `Some(version)` when an app with `bundle_id` is registered; the version is
/// `None` when its `Info.plist` has none.
pub fn find(bundle_id: &str) -> Option<Option<String>> {
    let id = cf_string(bundle_id)?;
    // SAFETY: `id` is a live CFString; a null error pointer is allowed.
    let urls = Owned::new(unsafe {
        LSCopyApplicationURLsForBundleIdentifier(id.0, std::ptr::null_mut())
    })?;
    // SAFETY: `urls` is a live CFArray.
    if unsafe { CFArrayGetCount(urls.0) } < 1 {
        return None;
    }
    // SAFETY: index 0 exists; the URL is borrowed from `urls`, alive here.
    let url = unsafe { CFArrayGetValueAtIndex(urls.0, 0) };
    Some(version(url))
}

fn version(url: CFTypeRef) -> Option<String> {
    // SAFETY: `url` is a live CFURL borrowed from the array.
    let bundle = Owned::new(unsafe { CFBundleCreate(std::ptr::null(), url) })?;
    let key = cf_string("CFBundleShortVersionString")?;
    // SAFETY: both are live; the value is borrowed from the bundle (Get rule).
    let value = unsafe { CFBundleGetValueForInfoDictionaryKey(bundle.0, key.0) };
    // SAFETY: `value` is checked for null before asking its type.
    if value.is_null() || unsafe { CFGetTypeID(value) != CFStringGetTypeID() } {
        return None;
    }
    let mut buffer = [0 as c_char; 64];
    // SAFETY: `value` is a CFString; `buffer`'s real length is passed.
    let copied =
        unsafe { CFStringGetCString(value, buffer.as_mut_ptr(), buffer.len() as isize, UTF8) };
    if copied == 0 {
        return None;
    }
    // SAFETY: on success the buffer holds a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    text.to_str().ok().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_null_object_is_none_and_never_released() {
        assert!(Owned::new(std::ptr::null()).is_none());
    }

    #[test]
    fn an_unregistered_bundle_id_is_not_found() {
        assert_eq!(find("br.invalid.websign.no-such-app"), None);
    }
}
