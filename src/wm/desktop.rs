//! Virtual desktop helpers. See the [`EWMH spec`].
//!
//! [`EWMH spec`]: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm45643494463472

#![allow(dead_code)]

use std::sync::Arc;

use super::ewmh;
use super::layout;
use crate::prelude::*;

/// Structure containing all clients on a virtual desktop, or workspace.
///
/// Clients owned by this desktop will always need to be visible. For instance,
/// removing a client from this desktop will make it invisible, so it will need to be
/// manually re-mapped if needed. Owned windows will also have their geometry
/// managed by a provided [`layout::Layout`] manager.
pub struct Desktop {
    name: String,
    conn: Arc<xcb::Connection>,
    clients: Vec<xcb::Window>,
    layout_mgr: Box<dyn layout::Layout>,
    ewmh_mgr: Arc<ewmh::EWMHManager>,
}

impl Desktop {
    #[must_use]
    pub fn new(
        conn: Arc<xcb::Connection>,
        name: String,
        layout_mgr: Box<dyn layout::Layout>,
        ewmh_mgr: Arc<ewmh::EWMHManager>,
    ) -> Self {
        Self {
            name,
            conn,
            clients: vec![],
            layout_mgr,
            ewmh_mgr,
        }
    }

    /// Get the name of this desktop.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Get a stack of clients owned by this desktop. The active window is
    /// always on the top of the stack.
    pub fn get_clients(&self) -> &Vec<xcb::Window> {
        &self.clients
    }

    /// Push a window to the stack and focus it.
    pub fn focus(&mut self, client: xcb::Window) -> NerdResult<()> {
        // Push the client onto the top of the stack.
        if let Some(p) = self.clients.iter().position(|c| c == &client) {
            // If this desktop already holds the client,
            // move it to the front of the stack.
            let client = self.clients.remove(p);
            self.clients.insert(0, client);
            self.layout_mgr.configure(&self.clients);
        } else {
            self.clients.insert(0, client);
            self.layout_mgr.configure(&self.clients);
        }

        // Make sure the window is visible.
        xcb::map_window_checked(&self.conn, client).request_check()?;

        Ok(())
    }

    /// Remove a window from the stack, and unmap it.
    pub fn remove(&mut self, client: xcb::Window) -> NerdResult<()> {
        if let Some(p) = self.clients.iter().position(|c| c == &client) {
            self.clients.remove(p);
            self.layout_mgr.configure(&self.clients);
        }

        // Hide the window.
        xcb::unmap_window_checked(&self.conn, client).request_check()?;

        Ok(())
    }

    /// Show all the clients owned by this desktop.
    pub fn show(&mut self) -> NerdResult<()> {
        for client in self.clients.iter().rev() {
            xcb::map_window_checked(&self.conn, *client).request_check()?;
        }

        Ok(())
    }

    /// Hide all the clients owned by this desktop.
    pub fn hide(&self) -> NerdResult<()> {
        for client in self.clients.iter().rev() {
            xcb::unmap_window_checked(&self.conn, *client).request_check()?;
        }

        Ok(())
    }
}
