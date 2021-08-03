//! Workspace management utilities.

pub mod client;
pub mod layout;

use std::rc::Rc;

use log::*;
use nerdwm_x11::context::DisplayContext;
use nerdwm_x11::window::Window;
use nerdwm_x11::xlib;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use client::ClientWindow;

/// Current state of inputs.
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
    prev_mouse: (i32, i32),
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
    fn get_client(&self, xid: u64) -> Option<usize> {
        self.clients
            .iter()
            .position(|w| w.internal.get_xid() == xid)
    }

    /// Get client position in stack from frame xid.
    fn get_client_from_frame(&self, xid: u64) -> Option<usize> {
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

    pub fn on_window_create(&mut self, event: xlib::XCreateWindowEvent) {
        debug!("Window Created {:x?}", event.window);
    }

    pub fn on_window_destroy(&mut self, event: xlib::XDestroyWindowEvent) {
        if let Some(pos) = self.get_client(event.window) {
            self.clients.remove(pos).destroy(&self.context, false);
            trace!("Destroyed frame");
        }
        debug!("Destroyed window {:x?}", event.window);
    }

    pub fn window_configure_request(&mut self, event: xlib::XConfigureRequestEvent) {
        let mut changes = xlib::XWindowChanges {
            x: event.x,
            y: event.y,
            width: event.width,
            height: event.height,
            border_width: event.border_width,
            sibling: event.above,
            stack_mode: event.detail,
        };

        // If a window exists, reconfigure its frame as well to accommodate resizing/etc.
        if let Some(pos) = self.get_client(event.window) {
            let window = self.clients[pos];
            let mut frame_changes = changes;
            window
                .frame
                .configure(&self.context, &mut frame_changes, event.value_mask as u32);
            trace!("Configured frame");
        }

        let window = Window::from_xid(event.window);

        window.configure(&self.context, &mut changes, event.value_mask as u32);

        debug!("Configured window {:x?}", event.window);
    }

    pub fn window_map_request(&mut self, event: xlib::XMapRequestEvent) {
        if self.get_client(event.window).is_none() {
            self.push(Window::from_xid(event.window))
        }
        debug!("Mapped window {:x?}", event.window);
    }

    pub fn on_window_unmap(&mut self, event: xlib::XUnmapEvent) {
        if let Some(pos) = self.get_client(event.window) {
            self.clients.remove(pos).destroy(&self.context, true);
            trace!("Destroyed frame");
        }
        debug!("Unmapped window {:x?}", event.window);
    }

    pub fn on_button_press(&mut self, event: xlib::XButtonPressedEvent) {
        // Event will happen on the frame
        self.prev_mouse = (event.x_root, event.y_root);
        if let Some(pos) = self.get_client_from_frame(event.subwindow) {
            trace!("Got event window at index {}", pos);

            for bind in &self.config.mousebinds {
                if event.button == u32::from(bind.bind) && event.state == bind.get_mask()
                // state -> modifier mask
                {
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

    pub fn on_button_release(&mut self, _event: xlib::XButtonReleasedEvent) {
        self.mode = Mode::None;
    }

    pub fn on_pointer_move(&mut self, event: xlib::XMotionEvent) {
        match self.mode {
            Mode::Move(client) => {
                let properties = client.frame.get_properties(&self.context);
                let mut changes = xlib::XWindowChanges {
                    x: properties.x + (event.x_root - self.prev_mouse.0),
                    y: properties.y + (event.y_root - self.prev_mouse.1),
                    width: 0,
                    height: 0,
                    border_width: 0,
                    sibling: 0,
                    stack_mode: 0,
                };
                client
                    .frame
                    .configure(&self.context, &mut changes, (xlib::CWX | xlib::CWY) as u32);
            }
            Mode::Resize(client) => {
                let properties = client.internal.get_properties(&self.context);
                let mut changes = xlib::XWindowChanges {
                    x: 0,
                    y: 0,
                    width: properties.width + (event.x_root - self.prev_mouse.0),
                    height: properties.height + (event.y_root - self.prev_mouse.1),
                    border_width: 0,
                    sibling: 0,
                    stack_mode: 0,
                };
                let mut frame_changes = changes;
                client.internal.configure(
                    &self.context,
                    &mut changes,
                    (xlib::CWWidth | xlib::CWHeight) as u32,
                );
                client.frame.configure(
                    &self.context,
                    &mut frame_changes,
                    (xlib::CWWidth | xlib::CWHeight) as u32,
                );
            }
            _ => {}
        }
        self.prev_mouse = (event.x_root, event.y_root);
    }
}
