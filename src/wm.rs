use log::{debug, info};
use x11_dl::xlib;

use crate::display_manager::DisplayManager;
use crate::window;

/// Manage windows, their properties, and decorations.
pub struct WindowManager {
    context: DisplayManager,
    windows: Vec<window::Window>,
}

impl WindowManager {
    /// Creates a new window manager, and connection to the X server.
    pub fn new() -> Self {
        let context = DisplayManager::new();
        context.init_root();

        let mut windows = vec![];

        // Create handles for existing windows
        context.grab_server();
        for w in context.get_windows(context.root) {
            windows.push(window::Window::new(&context, w))
        }
        context.ungrab_server();

        Self { context, windows }
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
                        .find(|w| w.id == configure_request.window)
                    {
                        let mut frame_config = configure_request.clone();
                        frame_config.window = window.frame.id;
                        self.context.configure_window(&mut changes, &frame_config)
                    }

                    self.context
                        .configure_window(&mut changes, &configure_request);
                }
                // Window Map Request
                xlib::MapRequest => {
                    self.windows.push(window::Window::new(
                        &self.context,
                        unsafe { event.map_request }.window,
                    ));
                }
                // On Window Unmap
                xlib::UnmapNotify => {
                    let unmap_event = unsafe { event.unmap };

                    if let Some(pos) = self.windows.iter().position(|w| w.id == unmap_event.window)
                    {
                        self.windows.remove(pos).destroy(&self.context);
                    }
                }
                _ => {}
            }
        }
    }
}
