// Copyright 2013-2015, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

use std::process;
use std::thread;
use glib_ffi::{self, gboolean, gpointer};
use translate::ToGlib;

/// Continue calling the closure in the future iterations or drop it.
///
/// This is the return type of `idle_add` and `timeout_add` closures.
///
/// `Continue(true)` keeps the closure assigned, to be rerun when appropriate.
///
/// `Continue(false)` disconnects and drops it.
pub struct Continue(pub bool);

impl ToGlib for Continue {
    type GlibType = gboolean;

    #[inline]
    fn to_glib(&self) -> gboolean {
        self.0.to_glib()
    }
}

/// Unwinding propagation guard. Aborts the process if destroyed while
/// panicking.
pub struct CallbackGuard(());

impl CallbackGuard {
    pub fn new() -> CallbackGuard {
        CallbackGuard(())
    }
}

impl Drop for CallbackGuard {
    fn drop(&mut self) {
        if thread::panicking() {
            process::exit(101);
        }
    }
}

unsafe extern "C" fn trampoline<F>(func: gpointer) -> gboolean
where F: FnMut() -> Continue + Send + 'static {
    let _guard = CallbackGuard::new();
    let func = &mut *(func as *mut F);
    func().to_glib()
}

unsafe extern "C" fn destroy<T>(ptr: gpointer) {
    let _guard = CallbackGuard::new();
    Box::<T>::from_raw(ptr as *mut _);
}

/// Adds a closure to be called by the default main loop when it's idle.
///
/// `func` will be called repeatedly until it returns `Continue(false)`.
///
/// The default main loop almost always is the main loop of the main thread.
/// Thus the closure is called on the main thread.
pub fn idle_add<F>(func: F) -> u32
where F: FnMut() -> Continue + Send + 'static {
    unsafe {
        glib_ffi::g_idle_add_full(glib_ffi::G_PRIORITY_DEFAULT_IDLE, Some(trampoline::<F>),
            Box::into_raw(Box::new(func)) as *mut _, Some(destroy::<F>))
    }
}

/// Adds a closure to be called by the default main loop at regular intervals
/// with millisecond granularity.
///
/// `func` will be called repeatedly every `interval` milliseconds until it
/// returns `Continue(false)`. Precise timing is not guaranteed, the timeout may
/// be delayed by other events. Prefer `timeout_add_seconds` when millisecond
/// precision is not necessary.
///
/// The default main loop almost always is the main loop of the main thread.
/// Thus the closure is called on the main thread.
pub fn timeout_add<F>(interval: u32, func: F) -> u32
where F: FnMut() -> Continue + Send + 'static {
    unsafe {
        glib_ffi::g_timeout_add_full(glib_ffi::G_PRIORITY_DEFAULT, interval, Some(trampoline::<F>),
            Box::into_raw(Box::new(func)) as *mut _, Some(destroy::<F>))
    }
}

/// Adds a closure to be called by the default main loop at regular intervals
/// with second granularity.
///
/// `func` will be called repeatedly every `interval` seconds until it
/// returns `Continue(false)`. Precise timing is not guaranteed, the timeout may
/// be delayed by other events.
///
/// The default main loop almost always is the main loop of the main thread.
/// Thus the closure is called on the main thread.
pub fn timeout_add_seconds<F>(interval: u32, func: F) -> u32
where F: FnMut() -> Continue + Send + 'static {
    unsafe {
        glib_ffi::g_timeout_add_seconds_full(glib_ffi::G_PRIORITY_DEFAULT, interval,
            Some(trampoline::<F>), Box::into_raw(Box::new(func)) as *mut _, Some(destroy::<F>))
    }
}
