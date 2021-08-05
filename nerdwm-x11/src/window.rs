//! X window wrapper.

use crate::context::{self, DisplayContext};

/// Structure and methods containing an X window.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    xid: u32,
}

impl Window {
    /// Create a new window with the given properties.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        context: &DisplayContext,
        parent: &Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
    ) -> Self {
        let xid = context.get_connection().generate_id();

        xcb::create_window_checked(
            context.get_connection(),
            xcb::COPY_FROM_PARENT as u8,
            xid,
            parent.get_xid(),
            x,
            y,
            width,
            height,
            border_width,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            xcb::COPY_FROM_PARENT,
            &[(0, 0)],
        );

        Self { xid }
    }

    /// Create a window from an existing X window.
    pub fn from_xid(xid: context::XID) -> Self {
        Self { xid }
    }

    /// Get XID of the window.
    pub fn get_xid(&self) -> context::XID {
        self.xid
    }

    /// Make the window visible.
    pub fn map(&self, context: &DisplayContext) {
        xcb::map_window_checked(context.get_connection(), self.xid);
    }

    /// Make the window invisible.
    pub fn unmap(&self, context: &DisplayContext) {
        xcb::unmap_window_checked(context.get_connection(), self.xid);
    }

    /// Raise the window.
    pub fn raise(&self, context: &DisplayContext) {
        self.configure(
            context,
            &[(xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE)],
        );
    }

    /// Add or remove window from the save set.
    ///
    /// The save-set is a list of windows, usually maintained by the window manager,
    /// but including only windows created by other clients. If the window manager dies,
    /// all windows listed in the save-set will be reparented back to their closest
    /// living ancestor if they were reparented in the first place and mapped if the
    /// window manager has unmapped them so that it could map an icon.
    pub fn set_save_set(&self, context: &DisplayContext, saved: bool) {
        xcb::change_save_set_checked(context.get_connection(), saved as u8, self.xid);
    }

    /// Request the X server to report these events.
    ///
    /// The events will be reported relative to the window they occurred on
    /// - meaning, when the event occurs, its `window` attribute will be the
    /// window is occurred on.
    pub fn set_event_mask(&self, context: &DisplayContext, mask: u32) {
        self.set_attribute(context, &[(xcb::CW_EVENT_MASK, mask)]);
    }

    /// Get list of child windows.
    pub fn get_tree<'a>(&self, context: &'a DisplayContext) -> xcb::QueryTreeCookie<'a> {
        xcb::query_tree_unchecked(context.get_connection(), self.xid)
    }

    /// Get dimensions, and position of window.
    pub fn get_geometry<'a>(&self, context: &'a DisplayContext) -> xcb::GetGeometryCookie<'a> {
        xcb::get_geometry(&context.get_connection(), self.xid)
    }

    /// Get all window attributes.
    pub fn get_attributes<'a>(
        &self,
        context: &'a DisplayContext,
    ) -> xcb::GetWindowAttributesCookie<'a> {
        xcb::get_window_attributes(context.get_connection(), self.xid)
    }

    /// Change window attributes.
    /// Each value must be a pair of (`ATTRIBUTE`, `VALUE`).
    pub fn set_attribute(&self, context: &DisplayContext, values: &[(u32, u32)]) {
        xcb::change_window_attributes_checked(context.get_connection(), self.xid, &values);
    }

    // TODO: properties
    // Properties are for example the window title (WM_NAME) or its minimum size (WM_NORMAL_HINTS).
    // Protocols such as EWMH also use properties - for example EWMH defines the window title, encoded as UTF-8 string, in the _NET_WM_NAME property.

    /// Configure window details such as size, position, border width and stacking order.
    pub fn configure(&self, context: &DisplayContext, values: &[(u16, u32)]) {
        xcb::configure_window_checked(context.get_connection(), self.xid, &values);
    }

    /// Change the border width.
    pub fn set_border_width(&self, context: &DisplayContext, width: u32) {
        self.configure(context, &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, width)])
    }

    /// Change the `CW_BORDER_PIXEL` attribute of the window.
    pub fn set_border_color(&self, context: &DisplayContext, color: u32) {
        self.set_attribute(context, &[(xcb::CW_BORDER_PIXEL, color)]);
    }

    /// Reparent the window to another window.
    pub fn reparent(&self, context: &DisplayContext, parent: &Window) {
        xcb::reparent_window_checked(context.get_connection(), self.xid, parent.get_xid(), 0, 0);
    }

    /// Send `WM_DELETE_WINDOW` to the window.
    pub fn kill(&self, _context: &DisplayContext) {}

    /// Destroy the window.
    pub fn destroy(self, context: &DisplayContext) {
        xcb::destroy_window_checked(context.get_connection(), self.xid);
    }
}
