use log::*;
use x11_dl::xlib;

use crate::client;
use crate::display_context::DisplayContext;
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
    windows: Vec<client::ClientWindow>,
}

impl WindowManager {
    /// Creates a new window manager, and connection to the X server.
    pub fn new() -> Self {
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
        context.flush();
        context.set_error_callback(Some(on_x_error));

        // Add existing windows to client list
        let mut windows = vec![];

        // Create handles for existing windows
        context.grab_server();
        for w in root.get_windows(&context) {
            let internal = window::Window::from_xid(w);
            let properties = internal.get_properties(&context);

            let frame = window::Window::create(
                &context,
                &root,
                properties.x,
                properties.y,
                properties.width as u32,
                properties.height as u32,
                5,
                0xffffff,
                0x111111,
            );

            frame.set_save_set(&context, true);
            internal.reparent(&context, &frame);

            frame.map(&context);
            internal.map(&context);

            let client_window = client::ClientWindow { internal, frame };

            windows.push(client_window)
        }
        context.ungrab_server();

        Self {
            context,
            root,
            windows,
        }
    }

    /// Run the event loop.
    pub fn run(&mut self) {
        loop {
            let event = self.context.get_next_event();

            debug!("Event {:?}", event);

            match event.get_type() {
                // On Window Create
                xlib::CreateNotify => info!("Window Created"),
                // Window Properties Change
                xlib::ConfigureRequest => {
                    let configure_request = unsafe { event.configure_request };

                    let mut changes = xlib::XWindowChanges {
                        x: configure_request.x,
                        y: configure_request.y,
                        width: configure_request.width,
                        height: configure_request.height,
                        border_width: configure_request.border_width,
                        sibling: configure_request.above,
                        stack_mode: configure_request.detail,
                    };

                    // If a window exists, reconfigure its frame as well to accomodate resizing/etc.
                    if let Some(window) = self
                        .windows
                        .iter()
                        .find(|w| w.internal.get_xid() == configure_request.window)
                    {
                        let mut frame_changes = changes;
                        window.frame.configure(
                            &self.context,
                            &mut frame_changes,
                            configure_request.value_mask as u32,
                        )
                    }

                    let window = window::Window::from_xid(configure_request.window);
                    window.configure(
                        &self.context,
                        &mut changes,
                        configure_request.value_mask as u32,
                    );
                }
                // Window Map Request
                xlib::MapRequest => {
                    let internal = window::Window::from_xid(unsafe { event.map_request }.window);
                    let properties = internal.get_properties(&self.context);

                    let frame = window::Window::create(
                        &self.context,
                        &self.root,
                        properties.x,
                        properties.y,
                        properties.width as u32,
                        properties.height as u32,
                        5,
                        0xffffff,
                        0x111111,
                    );

                    frame.set_save_set(&self.context, true);
                    internal.reparent(&self.context, &frame);

                    frame.map(&self.context);
                    internal.map(&self.context);

                    let client_window = client::ClientWindow { internal, frame };

                    self.windows.push(client_window)
                }
                // On Window Unmap
                xlib::UnmapNotify => {
                    let unmap_event = unsafe { event.unmap };

                    if let Some(pos) = self
                        .windows
                        .iter()
                        .position(|w| w.internal.get_xid() == unmap_event.window)
                    {
                        let client = self.windows.remove(pos);
                        client.frame.unmap(&self.context);
                        client.internal.reparent(&self.context, &self.root);
                        client.frame.set_save_set(&self.context, false);
                        client.frame.destroy(&self.context);
                    }
                }
                _ => {}
            }
        }
    }
}
