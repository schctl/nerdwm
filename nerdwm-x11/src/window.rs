//! X window wrapper.

use x11_dl::xlib;

use crate::context::DisplayContext;

/// Structure and methods containing an X window.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    xid: u64,
}

impl Window {
    /// Create a new window with the given properties.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        context: &DisplayContext,
        parent: &Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        border_color: u64,
        background_color: u64,
    ) -> Self {
        Self {
            xid: unsafe {
                (context.get_raw_context().XCreateSimpleWindow)(
                    context.get_connection(),
                    parent.get_xid(),
                    x,
                    y,
                    width,
                    height,
                    border_width,
                    border_color,
                    background_color,
                )
            },
        }
    }

    /// Create a window from an existing X window.
    pub fn from_xid(xid: u64) -> Self {
        Self { xid }
    }

    /// Get XID of the window.
    pub fn get_xid(&self) -> u64 {
        self.xid
    }

    /// Make the window visible.
    pub fn map(&self, context: &DisplayContext) {
        unsafe { (context.get_raw_context().XMapWindow)(context.get_connection(), self.xid) };
    }

    /// Make the window invisible.
    pub fn unmap(&self, context: &DisplayContext) {
        unsafe { (context.get_raw_context().XUnmapWindow)(context.get_connection(), self.xid) };
    }

    /// Raise the window.
    pub fn raise(&self, context: &DisplayContext) {
        unsafe { (context.get_raw_context().XRaiseWindow)(context.get_connection(), self.xid) };
    }

    /// Add or remove window from the save set.
    pub fn set_save_set(&self, context: &DisplayContext, saved: bool) {
        if saved {
            unsafe {
                (context.get_raw_context().XAddToSaveSet)(context.get_connection(), self.xid)
            };
        } else {
            unsafe {
                (context.get_raw_context().XRemoveFromSaveSet)(context.get_connection(), self.xid)
            };
        }
    }

    /// Set input event mask.
    pub fn set_event_mask(&self, context: &DisplayContext, mask: i64) {
        unsafe {
            (context.get_raw_context().XSelectInput)(context.get_connection(), self.xid, mask)
        };
    }

    /// Get list of child windows.
    pub fn get_children(&self, context: &DisplayContext) -> Vec<u64> {
        unsafe {
            let mut returned_root: u64 = std::mem::zeroed();
            let mut returned_parent: u64 = std::mem::zeroed();

            let mut num_windows: u32 = std::mem::zeroed();
            let mut window_list: *mut u64 = std::mem::zeroed();

            assert_ne!(
                (context.get_raw_context().XQueryTree)(
                    context.get_connection(),
                    self.xid,
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

    /// Get window properties.
    pub fn get_properties(&self, context: &DisplayContext) -> xlib::XWindowAttributes {
        let mut window_properties: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        unsafe {
            (context.get_raw_context().XGetWindowAttributes)(
                context.get_connection(),
                self.xid,
                &mut window_properties,
            )
        };
        window_properties
    }

    /// Change window properties.
    pub fn set_properties(
        &self,
        context: &DisplayContext,
        properties: &mut xlib::XSetWindowAttributes,
        value_mask: u64,
    ) {
        unsafe {
            (context.get_raw_context().XChangeWindowAttributes)(
                context.get_connection(),
                self.xid,
                value_mask,
                properties,
            )
        };
    }

    /// Configure window attributes such as position, size, and border.
    pub fn configure(
        &self,
        context: &DisplayContext,
        changes: &mut xlib::XWindowChanges,
        value_mask: u32,
    ) {
        unsafe {
            (context.get_raw_context().XConfigureWindow)(
                context.get_connection(),
                self.xid,
                value_mask,
                changes,
            )
        };
    }

    /// Change the border width.
    pub fn set_border_width(&self, context: &DisplayContext, width: u32) {
        unsafe {
            (context.get_raw_context().XSetWindowBorderWidth)(
                context.get_connection(),
                self.xid,
                width,
            )
        };
    }

    /// Change the border color.
    pub fn set_border_color(&self, context: &DisplayContext, color: u64) {
        unsafe {
            (context.get_raw_context().XSetWindowBorder)(context.get_connection(), self.xid, color)
        };
    }

    /// Reparent the window to another window.
    pub fn reparent(&self, context: &DisplayContext, parent: &Window) {
        unsafe {
            (context.get_raw_context().XReparentWindow)(
                context.get_connection(),
                self.xid,
                parent.get_xid(),
                0,
                0,
            )
        };
    }

    // /// Change the position of the window relative to its parent.
    // pub fn move_resize()

    /// Send WM_DELETE_WINDOW or forcefully kill client.
    pub fn kill(&self, _context: &DisplayContext) {}

    /// Destroy the window.
    pub fn destroy(self, context: &DisplayContext) {
        unsafe { (context.get_raw_context().XDestroyWindow)(context.get_connection(), self.xid) };
    }
}
