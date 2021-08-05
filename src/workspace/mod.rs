//! Workspace management utilities.

pub mod client;
pub mod layout;

use std::rc::Rc;

use log::*;
use nerdwm_x11::context::DisplayContext;
use nerdwm_x11::window::Window;
use nerdwm_x11::xcb;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use client::ClientWindow;

// TODO: collapse mode and action?

/// Current state of inputs.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Regular mode.
    None,
    /// Actions affect window position.
    Move(ClientWindow),
    /// Actions affect window size.
    Resize(ClientWindow),
}

/// WM actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Action {
    None,

    WindowMove,
    WindowResize,
    WindowClose,
    WindowFocus,
}

/// Workspace manager.
pub struct Workspace {
    /// Display context
    context: Rc<DisplayContext>,
    /// Name of the workspace.
    pub tag: String,
    /// Window stack.
    clients: Vec<ClientWindow>,
    /// Layout configuration
    config: Config,
    /// Layout manager
    layout_manager: Box<dyn layout::LayoutManager>,
    /// Save previous mouse position to calculate deltas
    prev_mouse: (i16, i16),
    /// Input mode
    mode: Mode,
}

impl Workspace {
    /// Create a new workspace.
    pub fn new(
        tag: String,
        context: Rc<DisplayContext>,
        config: Config,
        layout_manager: Box<dyn layout::LayoutManager>,
    ) -> Self {
        Self {
            context,
            tag,
            clients: vec![],
            config,
            layout_manager,
            prev_mouse: (0, 0),
            mode: Mode::None,
        }
    }

    /// Push a window onto the stack.
    pub fn push(&mut self, window: Window) {
        let client = ClientWindow::from_window(&self.context, window, &self.config.layout.border);
        client.frame.map(&self.context);
        client.internal.map(&self.context);
        self.focus_update(client);

        self.layout_manager.config(&self.clients);
    }

    /// Delete a window from the stack.
    pub fn pop(&mut self, index: usize) -> Window {
        let client = self.clients.remove(index);
        let new_focused = self.clients.remove(0);
        self.focus_update(new_focused);
        client.destroy(&self.context, false)
    }

    /// Get client position in stack if it exists.
    fn get_client(&self, xid: u32) -> Option<usize> {
        self.clients
            .iter()
            .position(|w| w.internal.get_xid() == xid)
    }

    /// Get client position in stack from frame xid.
    fn get_client_from_frame(&self, xid: u32) -> Option<usize> {
        self.clients.iter().position(|w| w.frame.get_xid() == xid)
    }

    /// Focus first window in the stack, and set attributes.
    fn focus_update(&mut self, client: ClientWindow) {
        client.frame.raise(&self.context);
        client
            .frame
            .set_border_width(&self.context, self.config.layout.border.width);
        client
            .frame
            .set_border_color(&self.context, self.config.layout.border.color);

        // Push the client to the front of the stack
        self.clients.insert(0, client);

        // Unfocus previously focused window
        if self.clients.len() > 1 {
            self.unfocus_update(1);
        }
    }

    /// Update unfocused window attributes.
    fn unfocus_update(&self, index: usize) {
        if self.clients.len() > index {
            self.clients[index]
                .frame
                .set_border_width(&self.context, self.config.layout.border_unfocused.width);
            self.clients[index]
                .frame
                .set_border_color(&self.context, self.config.layout.border_unfocused.color);
        }
    }

    // Specific event handlers
    // -----------------------

    pub fn on_window_create(&mut self, event: &xcb::CreateNotifyEvent) {
        trace!("Window Created {:x?}", unsafe { (*event.ptr).window });
    }

    pub fn on_window_destroy(&mut self, event: &xcb::DestroyNotifyEvent) {
        if let Some(pos) = self.get_client(unsafe { (*event.ptr).window }) {
            let win = self.clients.remove(pos).destroy(&self.context, false);
            trace!("Destroyed window {:x?}", win.get_xid());
        }
    }

    pub fn window_configure_request(&mut self, event: &xcb::ConfigureRequestEvent) {
        let changes: [(u16, u32); 7] = [
            (xcb::CONFIG_WINDOW_X as u16, event.x() as u32),
            (xcb::CONFIG_WINDOW_Y as u16, event.y() as u32),
            (xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32),
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                event.stack_mode() as u32,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32),
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                event.border_width() as u32,
            ),
        ];

        // If a window exists, reconfigure its frame as well to accommodate resizing/etc.
        if let Some(pos) = self.get_client(event.window()) {
            let window = self.clients[pos];
            window.frame.configure(&self.context, &changes);
            trace!("Configured frame");
        }

        let window = Window::from_xid(event.window());

        window.configure(&self.context, &changes);

        trace!("Configured window {:x?}", event.window());
    }

    pub fn window_map_request(&mut self, event: &xcb::MapRequestEvent) {
        if self.get_client(event.window()).is_none() {
            self.push(Window::from_xid(event.window()));
        }
        trace!("Mapped window {:x?}", event.window());
    }

    pub fn on_window_unmap(&mut self, event: &xcb::UnmapNotifyEvent) {
        if let Some(pos) = self.get_client(event.window()) {
            self.clients.remove(pos).destroy(&self.context, true);
            trace!("Destroyed frame");
        }
        trace!("Unmapped window {:x?}", event.window());
    }

    pub fn on_button_press(&mut self, event: &xcb::ButtonPressEvent) {
        // Event will happen on the frame
        self.prev_mouse = (event.root_x(), event.root_y());
        if let Some(pos) = self.get_client_from_frame(event.child()) {
            for bind in &self.config.mousebinds {
                if event.detail() == bind.bind as u8 && event.state() as u32 == bind.get_mask() {
                    match bind.action {
                        Action::WindowMove => self.mode = Mode::Move(self.clients[pos]),
                        Action::WindowResize => self.mode = Mode::Resize(self.clients[pos]),
                        _ => {}
                    }
                }
            }

            // Ignore window focus because the window will be focused anyway
            let client = self.clients.remove(pos);
            self.focus_update(client);
        }
    }

    pub fn on_button_release(&mut self, _event: &xcb::ButtonReleaseEvent) {
        self.mode = Mode::None;
    }

    pub fn on_pointer_move(&mut self, event: &xcb::MotionNotifyEvent) {
        match self.mode {
            Mode::Move(client) => {
                let properties = client
                    .frame
                    .get_geometry(&self.context)
                    .get_reply()
                    .unwrap();
                let changes: [(u16, u32); 2] = [
                    (
                        xcb::CONFIG_WINDOW_X as u16,
                        (properties.x() + (event.root_x() - self.prev_mouse.0 as i16)) as u32,
                    ),
                    (
                        xcb::CONFIG_WINDOW_Y as u16,
                        (properties.y() + (event.root_y() - self.prev_mouse.1 as i16)) as u32,
                    ),
                ];
                client.frame.configure(&self.context, &changes);
            }
            Mode::Resize(client) => {
                let properties = client
                    .frame
                    .get_geometry(&self.context)
                    .get_reply()
                    .unwrap();
                let changes: [(u16, u32); 2] = [
                    (
                        xcb::CONFIG_WINDOW_WIDTH as u16,
                        std::cmp::max(
                            (properties.width() as i16
                                + (event.root_x() - self.prev_mouse.0 as i16))
                                as u32,
                            5,
                        ),
                    ),
                    (
                        xcb::CONFIG_WINDOW_HEIGHT as u16,
                        std::cmp::max(
                            (properties.height() as i16
                                + (event.root_y() - self.prev_mouse.1 as i16))
                                as u32,
                            5,
                        ),
                    ),
                ];
                client.frame.configure(&self.context, &changes);
                client.internal.configure(&self.context, &changes);
            }
            _ => {}
        }
        self.prev_mouse = (event.root_x(), event.root_y());
    }
}
