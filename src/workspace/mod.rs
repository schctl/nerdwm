//! Workspace management utilities.

pub mod client;
pub mod layout;

use std::rc::Rc;

use log::*;
use x11_dl::xlib;

use crate::config::Config;
use crate::context::DisplayContext;
use crate::event;
use crate::window::Window;
use client::ClientWindow;

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
    mode: event::Mode,
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
            mode: event::Mode::None,
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
        if self.clients.len() == index + 1 {
            self.clients[index]
                .frame
                .set_border_width(&self.context, self.config.layout.border_unfocused.width);
            self.clients[index]
                .frame
                .set_border_color(&self.context, self.config.layout.border_unfocused.color);
        }
    }

    /// Propagate event to workspace.
    pub fn send_event(&mut self, event: event::Event) {
        match event {
            // On Window Create
            event::Event::WindowCreate(e) => debug!("Window Created {:x?}", e.window),
            // Window Properties Change
            event::Event::WindowConfigureRequest(configure_request) => {
                let mut changes = xlib::XWindowChanges {
                    x: configure_request.x,
                    y: configure_request.y,
                    width: configure_request.width,
                    height: configure_request.height,
                    border_width: configure_request.border_width,
                    sibling: configure_request.above,
                    stack_mode: configure_request.detail,
                };

                // If a window exists, reconfigure its frame as well to accommodate resizing/etc.
                if let Some(pos) = self.get_client(configure_request.window) {
                    let window = self.clients[pos];
                    let mut frame_changes = changes;
                    window.frame.configure(
                        &self.context,
                        &mut frame_changes,
                        configure_request.value_mask as u32,
                    );
                    trace!("Configured frame");
                }

                let window = Window::from_xid(configure_request.window);

                window.configure(
                    &self.context,
                    &mut changes,
                    configure_request.value_mask as u32,
                );

                debug!("Configured window {:x?}", configure_request.window);
            }
            // Window Map Request
            event::Event::WindowMapRequest(map_request) => {
                if self.get_client(map_request.window).is_none() {
                    self.push(Window::from_xid(map_request.window))
                }
                debug!("Mapped window {:x?}", map_request.window);
            }
            // On Window Unmap
            event::Event::WindowUnmap(unmap_event) => {
                if let Some(pos) = self.get_client(unmap_event.window) {
                    self.clients.remove(pos).destroy(&self.context, true);
                    trace!("Destroyed frame");
                }
                debug!("Unmapped window {:x?}", unmap_event.window);
            }
            event::Event::WindowDestroy(destroy_event) => {
                if let Some(pos) = self.get_client(destroy_event.window) {
                    self.clients.remove(pos).destroy(&self.context, false);
                    trace!("Destroyed frame");
                }
                debug!("Destroyed window {:x?}", destroy_event.window);
            }
            event::Event::ButtonPress(button_press) => {
                // Event will happen on the frame
                self.prev_mouse = (button_press.x_root, button_press.y_root);
                if let Some(pos) = self.get_client_from_frame(button_press.subwindow) {
                    trace!("Got event window at index {}", pos);

                    for bind in &self.config.mousebinds {
                        if button_press.button == u32::from(bind.bind) {
                            match bind.action {
                                event::Action::WindowMove => {
                                    self.mode = event::Mode::Move(self.clients[pos])
                                }
                                event::Action::WindowResize => {
                                    self.mode = event::Mode::Resize(self.clients[pos])
                                }
                                _ => {}
                            }
                        }
                    }

                    let client = self.clients.remove(pos);
                    self.focus_update(client);
                }
            }
            event::Event::PointerMotion(motion) => {
                match self.mode {
                    event::Mode::Move(client) => {
                        let properties = client.frame.get_properties(&self.context);
                        let mut changes = xlib::XWindowChanges {
                            x: properties.x + (motion.x_root - self.prev_mouse.0),
                            y: properties.y + (motion.y_root - self.prev_mouse.1),
                            width: 0,
                            height: 0,
                            border_width: 0,
                            sibling: 0,
                            stack_mode: 0,
                        };
                        client.frame.configure(
                            &self.context,
                            &mut changes,
                            (xlib::CWX | xlib::CWY) as u32,
                        );
                    }
                    event::Mode::Resize(client) => {
                        let properties = client.internal.get_properties(&self.context);
                        let mut changes = xlib::XWindowChanges {
                            x: 0,
                            y: 0,
                            width: properties.width + (motion.x_root - self.prev_mouse.0),
                            height: properties.height + (motion.y_root - self.prev_mouse.1),
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
                self.prev_mouse = (motion.x_root, motion.y_root)
            }
            event::Event::ButtonRelease(_button_release) => self.mode = event::Mode::None,
            _ => {}
        }
    }
}
