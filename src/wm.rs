//! Window Manager implementation.

use log::*;
use x11_dl::keysym;
use x11_dl::xlib;

use crate::client;
use crate::config;
use crate::display_context::DisplayContext;
use crate::event;
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
        };

        // Create handles for existing windows
        wm.context.grab_server();

        // Add existing windows to client list
        for w in root.get_windows(&wm.context) {
            wm.push_client(w);
            info!("Found window {:x?}", w);
        }

        wm.context.ungrab_server();

        wm.init_root();
        wm.context.flush();

        wm
    }

    /// Set focus onto one client.
    fn focus_client(&self, window: &client::ClientWindow) {
        for bind in &self.config.keybinds {
            self.context
                .grab_key(&window.internal, bind.bind.into(), bind.get_mask())
        }

        for bind in &self.config.mousebinds {
            self.context
                .grab_button(&window.internal, bind.bind.into(), bind.get_mask())
        }

        info!("Focused window.")
    }

    /// Get currently focused client.
    fn get_focused_client(&self) -> Option<&client::ClientWindow> {
        if self.clients.is_empty() {
            Some(&self.clients[0])
        } else {
            None
        }
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

        let client = client::ClientWindow { internal, frame };
        self.focus_client(&client);
        self.clients.insert(0, client);
    }

    /// Get client position in stack if it exists.
    pub fn get_client(&self, xid: u64) -> Option<usize> {
        self.clients
            .iter()
            .position(|w| w.internal.get_xid() == xid)
    }

    /// Run the event loop.
    pub fn run(&mut self) {
        loop {
            let event = self.context.get_next_event();

            debug!("Event [{:x?}]", event);

            info!("Clients: [{:?}]", self.clients);

            match event {
                // On Window Create
                event::Event::WindowCreate(e) => info!("Window Created {:x?}", e.window),
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
                        debug!("Configured frame");
                    }

                    let window = window::Window::from_xid(configure_request.window);

                    window.configure(
                        &self.context,
                        &mut changes,
                        configure_request.value_mask as u32,
                    );

                    info!("Configured window {:x?}", configure_request.window);
                }
                // Window Map Request
                event::Event::WindowMapRequest(map_request) => {
                    self.push_client(map_request.window);
                    info!("Mapped window {:x?}", map_request.window);
                }
                // On Window Unmap
                event::Event::WindowUnmap(unmap_event) => {
                    if let Some(pos) = self.get_client(unmap_event.window) {
                        let client = self.clients.remove(pos);
                        client.frame.unmap(&self.context);
                        client.internal.reparent(&self.context, &self.root);
                        client.frame.set_save_set(&self.context, false);
                        client.frame.destroy(&self.context);

                        debug!("Destroyed frame");
                    }

                    info!("Unmapped window {:x?}", unmap_event.window);
                }
                event::Event::WindowDestroy(destroy_event) => {
                    if let Some(pos) = self.get_client(destroy_event.window) {
                        let client = self.clients.remove(pos);
                        client.frame.unmap(&self.context);
                        client.frame.set_save_set(&self.context, false);
                        client.frame.destroy(&self.context);

                        debug!("Destroyed frame");
                    }

                    info!("Destroyed window {:x?}", destroy_event.window);
                }
                event::Event::ButtonPress(button_press) => {
                    info!("Button {:x} pressed!", button_press.button);
                    // Get the window the event occurred on
                    if let Some(pos) = self.get_client(button_press.window) {
                        let window = self.clients[pos];
                        info!("Found event window: {:?}", window);
                    }
                }
                event::Event::ButtonRelease(button_release) => {
                    info!("Button {:x} released!", button_release.button);
                }
                _ => {}
            }
        }
    }
}
