//! Window Manager implementation.

use std::rc::Rc;

use log::*;
use x11_dl::xlib;

use crate::config::Config;
use crate::context::DisplayContext;
use crate::window::Window;
use crate::workspace::{Workspace, layout};

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

/// Manage workspaces, and X server connection.
pub struct WindowManager {
    context: Rc<DisplayContext>,
    config: Config,
    // workspaces: Vec<Workspace>,
    active_workspace: Workspace,
}

impl WindowManager {
    /// Creates a new window manager, and connection to the X server.
    pub fn new(config: Config) -> Self {
        let context = Rc::new(DisplayContext::new());

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
            context: context.clone(),
            config: config.clone(),
            // workspaces: vec![],
            active_workspace: Workspace::new(
                "main".to_owned(),
                context,
                config,
                Box::new(layout::FloatingLayoutManager {}),
            ),
        };

        wm.init_root();
        wm.context.flush();

        // Create handles for existing windows
        wm.context.grab_server();

        // Add existing windows to client list
        for w in wm.context.get_default_root().get_children(&wm.context) {
            wm.active_workspace.push(Window::from_xid(w));
            debug!("Found window {:x?}", w);
        }

        wm.context.ungrab_server();

        wm
    }

    /// Configure the root window.
    fn init_root(&self) {
        let root = self.context.get_default_root();
        let root_mask = xlib::SubstructureRedirectMask
            | xlib::SubstructureNotifyMask
            | xlib::ButtonPressMask
            | xlib::ButtonReleaseMask
            | xlib::PointerMotionMask;

        let mut properties: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        properties.cursor = self.context.get_cursor(68);
        properties.event_mask = root_mask;

        root.set_properties(
            &self.context,
            &mut properties,
            xlib::CWCursor | xlib::CWEventMask,
        );
        root.set_event_mask(&self.context, root_mask);

        self.grab_binds(&root);

        debug!("Initialized root window");
    }

    /// Grab window management bindings.
    fn grab_binds(&self, window: &Window) {
        for bind in &self.config.keybinds {
            self.context
                .grab_key(window, bind.bind.into(), bind.get_mask())
        }

        for bind in &self.config.mousebinds {
            self.context
                .grab_button(window, bind.bind.into(), bind.get_mask())
        }

        trace!("Grabbed bindings for window: {:x}", window.get_xid());
    }

    /// Run the event loop.
    pub fn run(&mut self) {
        loop {
            let event = self.context.get_next_event();

            trace!("Event [{:x?}]", event);

            self.active_workspace.send_event(event);
        }
    }
}
