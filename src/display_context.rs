//! X11 wrapper.

use log::*;
use x11_dl::xlib;

use crate::window;

/// Safe wrapper around an X server connection.
pub struct DisplayContext {
    /// X context
    xlib: xlib::Xlib,
    /// Connection to the server
    display: *mut xlib::_XDisplay,
}

impl DisplayContext {
    /// Create a new connection to the X server.
    pub fn new() -> Self {
        // Initialize X
        let xlib = xlib::Xlib::open().expect("Could not connect to X Server");
        // Connection to X server
        let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };

        assert!(!display.is_null(), "Null pointer in display");

        info!("Connected to X server");

        Self { xlib, display }
    }

    /// Get raw xlib context.
    pub fn get_raw_context(&self) -> &xlib::Xlib {
        &self.xlib
    }

    /// Get connection id.
    pub fn get_connection(&self) -> *mut xlib::_XDisplay {
        self.display
    }

    /// Get default root window.
    pub fn get_default_root(&self) -> window::Window {
        window::Window::from_xid(unsafe { (self.xlib.XDefaultRootWindow)(self.display) })
    }

    /// Set an error callback for xlib.
    pub fn set_error_callback(
        &self,
        callback: Option<unsafe extern "C" fn(*mut xlib::_XDisplay, *mut xlib::XErrorEvent) -> i32>,
    ) {
        unsafe { (self.xlib.XSetErrorHandler)(callback) };
    }

    /// Disable requests on all other connections.
    pub fn grab_server(&self) {
        unsafe { (self.xlib.XGrabServer)(self.display) };
    }

    /// Allow request processing on other connections.
    pub fn ungrab_server(&self) {
        unsafe { (self.xlib.XUngrabServer)(self.display) };
    }

    /// Flush the X command queue.
    pub fn flush(&self) {
        unsafe { (self.xlib.XSync)(self.display, xlib::False) };
    }

    /// Set the input event mask for a window.
    pub fn set_event_mask(&self, window: u64, mask: i64) {
        unsafe { (self.xlib.XSelectInput)(self.display, window, mask) };
    }

    /// Get next X event.
    pub fn get_next_event(&self) -> xlib::XEvent {
        unsafe {
            debug!("{} Pending events", (self.xlib.XPending)(self.display));

            let mut event: xlib::XEvent = std::mem::zeroed();
            (self.xlib.XNextEvent)(self.display, &mut event);
            event
        }
    }
}
