//! Virtual desktop helpers. See the [`EWMH spec`].
//!
//! [`EWMH spec`]: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm45643494463472

use std::sync::Arc;

use super::actions;
use super::events;
use super::ewmh;
use super::layout;
use crate::prelude::*;

/// The "state" of the desktop. Processing of
/// events will depend on this.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    None,
    MovingWindow(xcb::Window),
    ResizingWindow(xcb::Window),
}

/// Structure containing all clients on a virtual desktop, or workspace.
///
/// Clients owned by this desktop will always need to be visible.
/// For instance, removing a client from this desktop will make it
/// invisible, so it will need to be manually re-mapped if needed.
/// Owned windows will also have their geometry managed by a provided
/// [`layout::Layout`] manager.
pub struct Desktop {
    name: String,
    conn: Arc<xcb::Connection>,
    clients: Vec<xcb::Window>,
    layout_mgr: Box<dyn layout::Layout>,
    ewmh_mgr: Arc<ewmh::EWMHManager>,
    // internal window stuff
    // ---------------------
    /// Last known mouse position.
    /// Used to determine scale of window resizing/movement.
    last_mouse: (i16, i16),
    mode: Mode,
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
            last_mouse: (0, 0),
            mode: Mode::None,
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
            self.layout_mgr.configure(&self.clients)?;
        } else {
            self.clients.insert(0, client);
            self.layout_mgr.configure(&self.clients)?;
        }

        // Make sure the window is visible.
        xcb::map_window_checked(&self.conn, client).request_check()?;
        self.ewmh_mgr.update_active_window(Some(client))?;
        self.ewmh_mgr.update_client_list(&self.clients[..])?;
        Ok(())
    }

    /// Remove a window from the stack, and unmap it.
    pub fn remove(&mut self, client: xcb::Window) -> NerdResult<()> {
        if let Some(p) = self.clients.iter().position(|c| c == &client) {
            self.clients.remove(p);
            self.layout_mgr.configure(&self.clients)?;
        }

        // Hide the window.
        xcb::unmap_window_checked(&self.conn, client).request_check()?;
        self.ewmh_mgr.update_client_list(&self.clients[..])?;
        Ok(())
    }

    /// Show all the clients owned by this desktop.
    pub fn show(&mut self) -> NerdResult<()> {
        for client in self.clients.iter().rev() {
            xcb::map_window_checked(&self.conn, *client).request_check()?;
        }
        self.ewmh_mgr.update_client_list(&self.clients[..])?;
        Ok(())
    }

    /// Hide all the clients owned by this desktop.
    pub fn hide(&self) -> NerdResult<()> {
        for client in self.clients.iter().rev() {
            xcb::unmap_window_checked(&self.conn, *client).request_check()?;
        }
        self.ewmh_mgr.update_client_list(&[])?;
        Ok(())
    }

    /// Execute an action, and reconfigure the layout.
    ///
    /// # Actions
    ///  - **[`actions::ActionType::WindowMove`]:
    ///    This action is processed when the event is a:
    ///     - Button Press
    ///       Starts moving the window the button was pressed on.
    ///     - Button Release
    ///       Stops moving the window.
    ///     - Pointer motion
    ///       Moves the window by the delta in the pointer position.
    pub fn do_action(&mut self, action: actions::Action) -> NerdResult<()> {
        match action.get_type() {
            // Window movement handler
            actions::ActionType::WindowMove => {
                self.move_handler(action.get_event())?;
            }
            _ => {}
        }
        self.layout_mgr.configure(&self.clients[..])?;
        Ok(())
    }

    /// Internal handler for moving windows.
    fn move_handler(&mut self, event: &events::Event) -> NerdResult<()> {
        if let Mode::MovingWindow(client) = self.mode {
            match event {
                // Move window by pointer delta
                events::Event::PointerMotion(e) => {
                    debug!("Moving window!");
                    let properties = xcb::get_geometry(&self.conn, client).get_reply()?;
                    let changes: [(u16, u32); 2] = [
                        (
                            xcb::CONFIG_WINDOW_X as u16,
                            (properties.x() + (e.root_x() - self.last_mouse.0 as i16)) as u32,
                        ),
                        (
                            xcb::CONFIG_WINDOW_Y as u16,
                            (properties.y() + (e.root_y() - self.last_mouse.1 as i16)) as u32,
                        ),
                    ];
                    xcb::configure_window(&self.conn, client, &changes).request_check()?;
                    self.last_mouse = (e.root_x(), e.root_y());
                }
                // Stop moving a window when a button is released.
                events::Event::ButtonRelease(e) => {
                    self.mode = Mode::None;
                    self.last_mouse = (e.root_x(), e.root_y());
                }
                _ => {}
            }
        } else if self.mode == Mode::None {
            // Start moving the window
            if let events::Event::ButtonPress(e) = event {
                self.mode = Mode::MovingWindow(e.event());
                self.last_mouse = (e.root_x(), e.root_y());
            }
        }

        Ok(())
    }
}
