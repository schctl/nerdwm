//! X server connection utilities.

use std::rc::Rc;

use log::*;

use crate::event;
use crate::keysym::KeySymbols;
use crate::window;

pub type XID = u32;

/// Utilities for communicating with the X server.
///
/// Objects that are represented with an `xid`, are defined
/// in their own structures and provide their own methods.
/// Everything else is provided by this.
pub struct DisplayContext {
    /// X connection
    connection: Rc<xcb::Connection>,
    /// Preferred screen number
    screen_number: i32,
    /// Keysymbols for this connection
    keysymbols: KeySymbols,
}

impl DisplayContext {
    /// Create a new connection to the X server.
    pub fn new() -> Self {
        // Connect to the X server
        let (connection, screen_number) =
            xcb::Connection::connect(None).expect("Could not connect to the X server.");

        let connection = Rc::new(connection);
        let keysymbols = KeySymbols::new(connection.clone());

        info!("Connected to X server");

        Self {
            connection,
            screen_number,
            keysymbols,
        }
    }

    /// Get internal xcb connection object.
    pub fn get_connection(&self) -> &xcb::Connection {
        &self.connection
    }

    /// Get default root window.
    pub fn get_default_root(&self) -> window::Window {
        window::Window::from_xid(
            self.connection
                .get_setup()
                .roots()
                .nth(self.screen_number as usize)
                .unwrap()
                .root(),
        )
    }

    /// Get key symbols for this connection.
    pub fn get_key_symbols(&self) -> &KeySymbols {
        &self.keysymbols
    }

    /// Disable request processing on all other connections.
    pub fn grab_server(&self) {
        xcb::grab_server(&self.connection);
    }

    /// Allow request processing on all other connections.
    pub fn ungrab_server(&self) {
        xcb::ungrab_server(&self.connection);
    }

    /// Flush the X command queue.
    pub fn flush(&self) {
        self.connection.flush();
    }

    /// Get next input event.
    pub fn get_next_event(&self) -> event::Event {
        self.connection.wait_for_event().unwrap().into()
    }

    /// Create a cursor.
    pub fn get_cursor(&self, cursor_id: u16) -> u32 {
        // https://xcb.freedesktop.org/tutorial/mousecursors/
        let font = self.connection.generate_id();
        xcb::open_font_checked(&self.connection, font, "cursor");

        let cursor = self.connection.generate_id();
        xcb::create_glyph_cursor_checked(
            &self.connection,
            cursor,
            font,
            font,
            cursor_id,
            cursor_id + 1,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        cursor
    }

    /// Passively grab keyboard key.
    pub fn grab_key(&self, window: &window::Window, key: u32, modifiers: u16) {
        xcb::grab_key_checked(
            &self.connection,
            true,
            window.get_xid(),
            modifiers,
            self.keysymbols.get_keycode(key).next().unwrap(),
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );
    }

    /// Release grab on keyboard key.
    pub fn ungrab_key(&self, window: &window::Window, key: u32, modifiers: u16) {
        xcb::ungrab_key_checked(&self.connection, key as u8, window.get_xid(), modifiers);
    }

    /// Passively grab mouse button from window.
    pub fn grab_button(&self, window: &window::Window, button: u8, modifiers: u16) {
        xcb::grab_button_checked(
            &self.connection,
            false, // owner events
            window.get_xid(),
            (xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE
                | xcb::EVENT_MASK_POINTER_MOTION) as u16, // event mask
            xcb::GRAB_MODE_ASYNC as u8, // pointer mode
            xcb::GRAB_MODE_ASYNC as u8, // keyboard mode
            0,                          // confine to window
            0,                          // cursor
            button,
            modifiers,
        );
    }

    /// Release grab on mouse button.
    pub fn ungrab_button(&self, window: &window::Window, button: u8, modifiers: u16) {
        xcb::ungrab_button_checked(&self.connection, button, window.get_xid(), modifiers);
    }

    /// Actively grab the mouse pointer.
    pub fn grab_pointer(&self, window: &window::Window, cursor: u32) {
        // https://tronche.com/gui/x/xlib/input/XGrabPointer.html
        xcb::grab_pointer(
            &self.connection,
            true, // owner events
            window.get_xid(),
            (xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE
                | xcb::EVENT_MASK_POINTER_MOTION) as u16, // event mask
            xcb::GRAB_MODE_ASYNC as u8, // pointer mode
            xcb::GRAB_MODE_ASYNC as u8, // keyboard mode
            0,                          // confine to window
            cursor,                     // cursor
            xcb::CURRENT_TIME,
        );
    }

    /// Release grab on mouse pointer.
    pub fn ungrab_pointer(&self) {
        xcb::ungrab_pointer_checked(&self.connection, xcb::CURRENT_TIME);
    }
}
