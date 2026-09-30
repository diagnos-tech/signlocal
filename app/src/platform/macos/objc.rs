//! The few Objective-C messages the platform layer sends (AppKit,
//! SecurityInterface), and the one class it defines (the `websign:` URL
//! event receiver), through the runtime's C API instead of a binding crate:
//! `objc_msgSend` cast to the exact signature of each method.
//!
//! Only scalar and pointer returns are used, so `objc_msgSend` is correct on
//! both arm64 and x86_64 (no `_stret`/`_fpret` variants needed).

use std::ffi::{CStr, c_char, c_void};

/// An Objective-C object or class pointer.
pub type Id = *mut c_void;
/// A selector (method name) registered with the runtime.
pub type Sel = *const c_void;

/// Objective-C `BOOL` as the ABI passes it (one byte on both architectures).
pub type ObjcBool = i8;

#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra_bytes: usize) -> Id;
    fn objc_registerClassPair(class: Id);
    fn class_addMethod(class: Id, name: Sel, imp: *const c_void, types: *const c_char) -> ObjcBool;
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

/// The class named `name`, when its framework is loaded.
pub fn class(name: &CStr) -> Option<Id> {
    // SAFETY: `name` is NUL-terminated; unknown names give null.
    let class = unsafe { objc_getClass(name.as_ptr()) };
    (!class.is_null()).then_some(class)
}

/// The selector named `name`.
pub fn selector(name: &CStr) -> Sel {
    // SAFETY: `name` is NUL-terminated; registering is idempotent.
    unsafe { sel_registerName(name.as_ptr()) }
}

/// Sends `method` (no arguments) to `receiver`.
///
/// # Safety
/// `receiver` is a live object (or nil) that responds to `method`, whose
/// return type is `R`.
pub unsafe fn send0<R>(receiver: Id, method: &CStr) -> R {
    // SAFETY: `objc_msgSend` must be called through a pointer of the
    // method's own signature, which the caller vouches for.
    let send: unsafe extern "C" fn(Id, Sel) -> R =
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // SAFETY: as documented by the caller.
    unsafe { send(receiver, selector(method)) }
}

/// Sends `method` with one argument.
///
/// # Safety
/// As [`send0`], with `method` taking one `A`.
pub unsafe fn send1<A, R>(receiver: Id, method: &CStr, a: A) -> R {
    // SAFETY: as in `send0`.
    let send: unsafe extern "C" fn(Id, Sel, A) -> R =
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // SAFETY: as documented by the caller.
    unsafe { send(receiver, selector(method), a) }
}

/// Sends `method` with two arguments.
///
/// # Safety
/// As [`send0`], with `method` taking an `A` and a `B`.
pub unsafe fn send2<A, B, R>(receiver: Id, method: &CStr, a: A, b: B) -> R {
    // SAFETY: as in `send0`.
    let send: unsafe extern "C" fn(Id, Sel, A, B) -> R =
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // SAFETY: as documented by the caller.
    unsafe { send(receiver, selector(method), a, b) }
}

/// Sends `method` with four arguments.
///
/// # Safety
/// As [`send0`], with `method` taking an `A`, a `B`, a `C` and a `D`.
pub unsafe fn send4<A, B, C, D, R>(receiver: Id, method: &CStr, a: A, b: B, c: C, d: D) -> R {
    // SAFETY: as in `send0`.
    let send: unsafe extern "C" fn(Id, Sel, A, B, C, D) -> R =
        unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
    // SAFETY: as documented by the caller.
    unsafe { send(receiver, selector(method), a, b, c, d) }
}

/// One method of a class defined at run time: its selector name, its
/// implementation and its type encoding (`v@:@` = returns void, takes self,
/// the selector and one object).
pub struct Method {
    pub name: &'static CStr,
    pub imp: *const c_void,
    pub types: &'static CStr,
}

/// Defines and registers the `NSObject` subclass `name` with `methods`;
/// `None` when the name is taken or `NSObject` is missing. Call it once per
/// name (a `OnceLock` in the caller).
///
/// # Safety
/// Each `imp` is an `extern "C"` function whose parameters are exactly
/// `(Id, Sel, …)` as its `types` encoding declares.
pub unsafe fn define_class(name: &CStr, methods: &[Method]) -> Option<Id> {
    let superclass = class(c"NSObject")?;
    // SAFETY: a live class and a NUL-terminated name; null when taken.
    let new = unsafe { objc_allocateClassPair(superclass, name.as_ptr(), 0) };
    if new.is_null() {
        return None;
    }
    for method in methods {
        // SAFETY: `new` is allocated and not yet registered (methods may be
        // added); the implementation matches its encoding (caller's promise).
        unsafe {
            class_addMethod(
                new,
                selector(method.name),
                method.imp,
                method.types.as_ptr(),
            )
        };
    }
    // SAFETY: allocated above and registered once.
    unsafe { objc_registerClassPair(new) };
    Some(new)
}

/// AppKit objects may only be used on the main thread.
pub fn on_main_thread() -> bool {
    // SAFETY: no preconditions.
    unsafe { libc::pthread_main_np() == 1 }
}

/// Releases the autoreleased objects created while it lives (callers may
/// run outside AppKit's own pool).
#[derive(Debug)]
pub struct AutoreleasePool(*mut c_void);

impl AutoreleasePool {
    pub fn new() -> Self {
        // SAFETY: no preconditions; popped once in `drop`.
        Self(unsafe { objc_autoreleasePoolPush() })
    }
}

impl Drop for AutoreleasePool {
    fn drop(&mut self) {
        // SAFETY: the token from this pool's push, popped once, in order
        // (the guard is scoped to one function).
        unsafe { objc_autoreleasePoolPop(self.0) }
    }
}
