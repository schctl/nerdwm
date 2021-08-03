//! X connection wrapper.

use log::*;
use x11_dl::xlib;

use crate::event;
use crate::window;

/// Safe wrapper around an X server connection.
pub struct DisplayContext {
    /// X context
    xlib: xlib::Xlib,
    /// Connection to the server
    display: *mut xlib::_XDisplay,
}

impl Default for DisplayContext {
    fn default() -> Self {
        Self::new()
    }
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

    /// Get next input event.
    pub fn get_next_event(&self) -> event::Event {
        unsafe {
            trace!("{} Pending events", (self.xlib.XPending)(self.display));

            let mut raw_event: xlib::XEvent = std::mem::zeroed();
            (self.xlib.XNextEvent)(self.display, &mut raw_event);
            raw_event.into()
        }
    }

    /// Create a cursor.
    /// See cursor definition from <https://tronche.com/gui/x/xlib/appendix/b/>
    pub fn get_cursor(&self, cursor: u32) -> u64 {
        unsafe { (self.xlib.XCreateFontCursor)(self.display, cursor) }
    }

    /// Passively grab keyboard key.
    pub fn grab_key(&self, window: &window::Window, key: u32, modifiers: u32) {
        unsafe {
            // https://tronche.com/gui/x/xlib/input/XGrabKey.html
            (self.xlib.XGrabKey)(
                self.display,
                (self.xlib.XKeysymToKeycode)(self.display, key as u64) as i32, // key code
                modifiers,                                                     // modifier mask
                window.get_xid(),                                              // grab window
                1,                                                             // owner events (?)
                xlib::GrabModeAsync, // process pointer events without freezing
                xlib::GrabModeAsync, // process keyboard events without freezing
            )
        };
    }

    /// Release grab on keyboard key.
    pub fn ungrab_key(&self, window: &window::Window, key: u32, modifiers: u32) {
        unsafe { (self.xlib.XUngrabKey)(self.display, key as i32, modifiers, window.get_xid()) };
    }

    /// Passively grab mouse button from window.
    pub fn grab_button(&self, window: &window::Window, button: u32, modifiers: u32) {
        unsafe {
            // https://tronche.com/gui/x/xlib/input/XGrabButton.html
            (self.xlib.XGrabButton)(
                self.display,
                button,           // mouse button
                modifiers,        // modifier mask
                window.get_xid(), // grab window
                0,                // owner events
                (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as u32, // event mask
                xlib::GrabModeAsync, // process pointer events without freezing
                xlib::GrabModeAsync, // process keyboard events without freezing
                0,                   // confine pointer to window
                0,                   // cursor to display
            )
        };
    }

    /// Release grab on mouse button.
    pub fn ungrab_button(&self, window: &window::Window, button: u32, modifiers: u32) {
        unsafe { (self.xlib.XUngrabButton)(self.display, button, modifiers, window.get_xid()) };
    }

    /// Actively grab the mouse pointer.
    pub fn grab_pointer(&self, window: &window::Window, cursor: u64) {
        unsafe {
            // https://tronche.com/gui/x/xlib/input/XGrabPointer.html
            (self.xlib.XGrabPointer)(
                self.display,
                window.get_xid(), // grab window
                1,                // owner events
                (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as u32, // event mask
                xlib::GrabModeAsync, // process pointer events without freezing
                xlib::GrabModeAsync, // process keyboard events without freezing
                0,                   // confine to window
                cursor,              // cursor to display
                xlib::CurrentTime,
            )
        };
    }

    /// Release grab on mouse pointer.
    pub fn ungrab_pointer(&self) {
        unsafe { (self.xlib.XUngrabPointer)(self.display, xlib::CurrentTime) };
    }
}
