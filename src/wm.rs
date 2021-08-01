//! Window Manager implementation.

use log::*;
use x11_dl::keysym;
use x11_dl::xlib;

use crate::client;
use crate::config;
use crate::display_context::DisplayContext;
use crate::event;
use crate::input;
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

/// Manage windows, their properties, and decorations.
pub struct WindowManager {
    context: DisplayContext,
    root: window::Window,
    config: config::Config,
    /// Stack of clients.
    /// Order of focus.
    clients: Vec<client::ClientWindow>,
    mode: input::Mode,
    previous_mouse_position: (i32, i32),
}

impl WindowManager {
    /// Creates a new window manager, and connection to the X server.
    pub fn new(config: config::Config) -> Self {
        let context = DisplayContext::new();

        // Startup
        let root = context.get_default_root();

        // WM check
        context.set_error_callback(Some(on_startup_error));

        // Inputs for root window.
        // Substructure redirection allows the WM to intercept
        // these events and handle them on its own.
        root.set_event_mask(
            &context,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        );

        context.set_error_callback(Some(on_x_error));
        context.flush();

        let mut wm = Self {
            context,
            root,
            config,
            clients: vec![],
            mode: input::Mode::None,
            previous_mouse_position: (0, 0),
        };

        wm.init_root();
        wm.context.flush();

        // Create handles for existing windows
        wm.context.grab_server();

        // Add existing windows to client list
        for w in wm.root.get_children(&wm.context) {
            wm.push_client(w);
            debug!("Found window {:x?}", w);
        }

        wm.context.ungrab_server();

        wm
    }

    /// Configure the root window.
    fn init_root(&self) {
        let root_mask = xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::ButtonPressMask
            | xlib::ButtonReleaseMask
            | xlib::PointerMotionMask;

        let mut properties: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        properties.cursor = self.context.get_cursor(68);
        properties.event_mask = root_mask;

        self.root.set_properties(
            &self.context,
            &mut properties,
            xlib::CWCursor | xlib::CWEventMask,
        );
        self.root.set_event_mask(&self.context, root_mask);

        self.grab_binds(&self.root);

        debug!("Initialized root window");
    }

    /// Grab window management bindings.
    fn grab_binds(&self, window: &window::Window) {
        for bind in &self.config.keybinds {
            self.context
                .grab_key(&window, bind.bind.into(), bind.get_mask())
        }

        for bind in &self.config.mousebinds {
            self.context
                .grab_button(&window, bind.bind.into(), bind.get_mask())
        }

        trace!("Grabbed bindings for window: {:x}", window.get_xid());
    }

    /// Push a window to the current stack.
    fn push_client(&mut self, window: u64) {
        let internal = window::Window::from_xid(window);
        let properties = internal.get_properties(&self.context);

        let frame = window::Window::create(
            &self.context,
            &self.root,
            properties.x,
            properties.y,
            properties.width as u32,
            properties.height as u32,
            self.config.border_width,
            self.config.border_color,
            0x111111,
        );

        frame.set_event_mask(
            &self.context,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        );
        frame.set_save_set(&self.context, true);
        frame.map(&self.context);

        internal.reparent(&self.context, &frame);
        internal.map(&self.context);
        self.grab_binds(&internal);

        let client = client::ClientWindow { internal, frame };
        self.clients.push(client);
    }

    /// Get client position in stack if it exists.
    pub fn get_client(&self, xid: u64) -> Option<usize> {
        self.clients
            .iter()
            .position(|w| w.internal.get_xid() == xid)
    }

    /// Get client position in stack from frame xid.
    pub fn get_client_from_frame(&self, xid: u64) -> Option<usize> {
        self.clients.iter().position(|w| w.frame.get_xid() == xid)
    }

    /// Run the event loop.
    pub fn run(&mut self) {
        loop {
            let event = self.context.get_next_event();

            trace!("Event [{:x?}]", event);
            trace!("Clients: [{:x?}]", self.clients);
            trace!("Mode: {:x?}", self.mode);

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

                    let window = window::Window::from_xid(configure_request.window);

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
                        self.push_client(map_request.window)
                    }
                    debug!("Mapped window {:x?}", map_request.window);
                }
                // On Window Unmap
                event::Event::WindowUnmap(unmap_event) => {
                    if let Some(pos) = self.get_client(unmap_event.window) {
                        let client = self.clients.remove(pos);
                        client.frame.unmap(&self.context);
                        client.internal.reparent(&self.context, &self.root);
                        client.frame.set_save_set(&self.context, false);
                        client.frame.destroy(&self.context);

                        trace!("Destroyed frame");
                    }

                    debug!("Unmapped window {:x?}", unmap_event.window);
                }
                event::Event::WindowDestroy(destroy_event) => {
                    if let Some(pos) = self.get_client(destroy_event.window) {
                        let client = self.clients.remove(pos);
                        client.frame.unmap(&self.context);
                        client.frame.set_save_set(&self.context, false);
                        client.frame.destroy(&self.context);

                        trace!("Destroyed frame");
                    }

                    debug!("Destroyed window {:x?}", destroy_event.window);
                }
                event::Event::ButtonPress(button_press) => {
                    // Event will happen on the frame
                    self.previous_mouse_position = (button_press.x_root, button_press.y_root);
                    if let Some(pos) = self.get_client_from_frame(button_press.subwindow) {
                        trace!("Got event window at index {}", pos);

                        for bind in &self.config.mousebinds {
                            if button_press.button == u32::from(bind.bind) {
                                match bind.action {
                                    event::Action::WindowMove => {
                                        self.mode = input::Mode::Move(self.clients[pos])
                                    }
                                    event::Action::WindowResize => {
                                        self.mode = input::Mode::Resize(self.clients[pos])
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                event::Event::PointerMotion(motion) => {
                    match self.mode {
                        input::Mode::Move(client) => {
                            let properties = client.frame.get_properties(&self.context);
                            let mut changes = xlib::XWindowChanges {
                                x: properties.x + (motion.x_root - self.previous_mouse_position.0),
                                y: properties.y + (motion.y_root - self.previous_mouse_position.1),
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
                        input::Mode::Resize(client) => {
                            let properties = client.internal.get_properties(&self.context);
                            let mut changes = xlib::XWindowChanges {
                                x: 0,
                                y: 0,
                                width: properties.width
                                    + (motion.x_root - self.previous_mouse_position.0),
                                height: properties.height
                                    + (motion.y_root - self.previous_mouse_position.1),
                                border_width: 0,
                                sibling: 0,
                                stack_mode: 0,
                            };
                            let mut frame_changes = changes.clone();
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
                    self.previous_mouse_position = (motion.x_root, motion.y_root)
                }
                event::Event::ButtonRelease(_button_release) => self.mode = input::Mode::None,
                _ => {}
            }
        }
    }
}
