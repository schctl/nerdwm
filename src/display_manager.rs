#![allow(dead_code)]

/// Display Manager
/// Only supports X11 for now.
use log::{debug, error, info};
use x11_dl::xlib;

use crate::window;

/// Occurs if another WM is running.
extern "C" fn on_startup_error(_display: *mut xlib::Display, error: *mut xlib::XErrorEvent) -> i32 {
    let error = unsafe { *error };
    error!("X Error [{}] - {}", error.type_, error.error_code);
    std::process::exit(-1);
}

/// Occurs when the X server raises an error.
extern "C" fn on_x_error(_display: *mut xlib::Display, error: *mut xlib::XErrorEvent) -> i32 {
    let error = unsafe { *error };
    error!("X Error [{}] - {}", error.type_, error.error_code);
    1
}

type XRoot = u64;

/// Safe wrapper around connection to X server.
pub struct DisplayManager {
    /// X context
    xlib: xlib::Xlib,
    /// Connection to the server
    display: *mut xlib::_XDisplay,
    /// Root window
    pub root: XRoot,
}

impl DisplayManager {
    /// Create a new display manager and initialize X.
    pub fn new() -> Self {
        // Initialize X
        let xlib = xlib::Xlib::open().expect("Could not connect to Xorg Server");
        // Connection to X server
        let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };

        assert!(!display.is_null(), "Null pointer in display");

        info!("Connected to X server");

        // Root X window
        let root: XRoot = unsafe { (xlib.XDefaultRootWindow)(display) };

        Self {
            xlib,
            display,
            root,
        }
    }

    /// Initialize the root window.
    pub fn init_root(&self) {
        unsafe {
            // WM check
            (self.xlib.XSetErrorHandler)(Some(on_startup_error));

            // Inputs for root window.
            // Substructure redirection allows the WM to intercept
            // these events and handle them on its own.
            self.set_event_mask(
                self.root,
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );

            self.flush_request_queue();

            // Error handler
            (self.xlib.XSetErrorHandler)(Some(on_x_error))
        };

        debug!("Initialized root window");
    }

    /// Disable requests on all other connecions.
    pub fn grab_server(&self) {
        unsafe { (self.xlib.XGrabServer)(self.display) };
    }

    /// Allow request processing on other connections.
    pub fn ungrab_server(&self) {
        unsafe { (self.xlib.XUngrabServer)(self.display) };
    }

    /// Flush the X command queue.
    pub fn flush_request_queue(&self) {
        unsafe { (self.xlib.XSync)(self.display, xlib::False) };
        debug!("Flushed request queue");
    }

    /// Set the input event mask for a window.
    pub fn set_event_mask(&self, window: u64, mask: i64) {
        unsafe { (self.xlib.XSelectInput)(self.display, window, mask) };
    }

    /// Get next X event.
    pub fn get_next_event(&self) -> xlib::XEvent {
        debug!("{} Pending events", unsafe {
            (self.xlib.XPending)(self.display)
        });

        let mut event: xlib::XEvent = unsafe { std::mem::zeroed() };
        unsafe {
            (self.xlib.XNextEvent)(self.display, &mut event);
        };
        event
    }

    /// Event thread to read events and convert them to native events.
    pub fn spawn_event_reader(&self) {}

    /// Get list of child windows.
    pub fn get_windows(&self, window: u64) -> Vec<u64> {
        unsafe {
            let mut returned_root: u64 = std::mem::zeroed();
            let mut returned_parent: u64 = std::mem::zeroed();

            let mut num_windows: u32 = std::mem::zeroed();
            let mut window_list: *mut u64 = std::mem::zeroed();

            assert_ne!(
                (self.xlib.XQueryTree)(
                    self.display,
                    window,
                    &mut returned_root,
                    &mut returned_parent,
                    &mut window_list,
                    &mut num_windows
                ),
                0
            );

            std::slice::from_raw_parts(window_list, num_windows as usize).to_owned()
        }
    }

    /// Create window.
    pub fn create_window(
        &self,
        properties: window::WindowProperties,
        background_color: u64,
        border_width: u32,
        border_color: u64,
    ) -> u64 {
        let win = unsafe {
            (self.xlib.XCreateSimpleWindow)(
                self.display,
                self.root,
                properties.x,
                properties.y,
                properties.width as u32,
                properties.height as u32,
                border_width,
                border_color,
                background_color,
            )
        };
        debug!("Created window {}", win);
        win
    }

    /// Destroy a window
    pub fn destroy_window(&self, window: u64) {
        unsafe { (self.xlib.XDestroyWindow)(self.display, window) };
        debug!("Destroyed window {}", window);
    }

    /// Get properties of a window.
    pub fn get_window_properties(&self, window: u64) -> window::WindowProperties {
        let mut window_properties: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        unsafe { (self.xlib.XGetWindowAttributes)(self.display, window, &mut window_properties) };
        window_properties
    }

    /// Make a window visible.
    pub fn map_window(&self, window: u64) {
        unsafe { (self.xlib.XMapWindow)(self.display, window) };
        debug!("Mapped window {}", window);
    }

    /// Make a window invisible.
    pub fn unmap_window(&self, window: u64) {
        unsafe { (self.xlib.XUnmapWindow)(self.display, window) };
        debug!("Unmapped window {}", window);
    }

    /// Reparent a window.
    pub fn reparent_window(&self, window: u64, parent: u64) {
        unsafe { (self.xlib.XReparentWindow)(self.display, window, parent, 0, 0) };
    }

    /// Add window to save set.
    pub fn add_to_save_set(&self, window: u64) {
        unsafe { (self.xlib.XAddToSaveSet)(self.display, window) };
    }

    /// Remove a window from the save set.
    pub fn remove_from_save_set(&self, window: u64) {
        unsafe { (self.xlib.XRemoveFromSaveSet)(self.display, window) };
    }

    /// Change window properties.
    pub fn configure_window(
        &self,
        changes: &mut xlib::XWindowChanges,
        request: &xlib::XConfigureRequestEvent,
    ) {
        unsafe {
            (self.xlib.XConfigureWindow)(
                self.display,
                request.window,
                request.value_mask as u32,
                changes,
            )
        };

        debug!("Configured window {}", request.window);
    }
}
