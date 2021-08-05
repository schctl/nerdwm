//! Window Manager implementation.

use std::rc::Rc;

use log::*;
use nerdwm_x11::context::DisplayContext;
use nerdwm_x11::window::Window;
use nerdwm_x11::xcb;
use nerdwm_x11::{event, input};

use crate::config::Config;
use crate::workspace::{layout, Workspace};

/// Manage workspaces, and X server connection.
pub struct WindowManager {
    context: Rc<DisplayContext>,
    config: Config,
    active_workspace: Workspace,
}

impl WindowManager {
    /// Creates a new window manager, and connection to the X server.
    pub fn new(config: Config) -> Self {
        let context = Rc::new(DisplayContext::new());

        // Startup
        let root = context.get_default_root();

        // WM check

        // Inputs for root window.
        // Substructure redirection allows the WM to intercept
        // these events and handle them on its own.
        root.set_event_mask(
            &context,
            xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        );

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

        // Create handles for existing windows
        wm.context.grab_server();

        // Add existing windows to client list
        for w in wm
            .context
            .get_default_root()
            .get_tree(&wm.context)
            .get_reply()
            .unwrap()
            .children()
            .iter()
            .map(|w| Window::from_xid(*w))
        {
            wm.active_workspace.push(w);
            debug!("Found window {:x?}", w);
        }

        wm.context.ungrab_server();

        info!("Initialized");

        wm
    }

    /// Configure the root window.
    fn init_root(&self) {
        let root = self.context.get_default_root();
        let root_mask = xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
            | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
            | xcb::EVENT_MASK_BUTTON_PRESS
            | xcb::EVENT_MASK_BUTTON_RELEASE
            | xcb::EVENT_MASK_POINTER_MOTION
            | xcb::EVENT_MASK_PROPERTY_CHANGE;

        let cursor = self.context.get_cursor(68);

        root.set_attribute(&self.context, &[(xcb::CW_CURSOR, cursor)]);
        root.set_event_mask(&self.context, root_mask);

        self.context
            .ungrab_key(&root, xcb::GRAB_ANY, input::ModifierMask::Any as u16);
        self.grab_binds(&root);

        debug!("Initialized root window");
    }

    /// Grab window management bindings.
    fn grab_binds(&self, window: &Window) {
        for bind in &self.config.keybinds {
            self.context
                .grab_key(window, bind.bind as u32, bind.get_mask() as u16);
        }

        self.context.flush();

        for bind in &self.config.mousebinds {
            self.context
                .grab_button(window, bind.bind as u8, bind.get_mask() as u16);
        }

        self.context.flush();

        trace!("Grabbed bindings for window: {:x}", window.get_xid());
    }

    /// Run the event loop.
    pub fn run(&mut self) {
        loop {
            self.context.flush();

            let event = self.context.get_next_event();

            // ignore events we don't care about
            match event {
                // handle these events - binds
                event::Event::ButtonPress(e) => self.active_workspace.on_button_press(&e),
                event::Event::ButtonRelease(e) => self.active_workspace.on_button_release(&e),
                // and let the active workspace handle the rest
                event::Event::WindowCreate(e) => self.active_workspace.on_window_create(&e),
                event::Event::WindowConfigureRequest(e) => {
                    self.active_workspace.window_configure_request(&e);
                }
                event::Event::WindowMapRequest(e) => self.active_workspace.window_map_request(&e),
                event::Event::WindowUnmap(e) => self.active_workspace.on_window_unmap(&e),
                event::Event::WindowDestroy(e) => self.active_workspace.on_window_destroy(&e),
                event::Event::PointerMotion(e) => self.active_workspace.on_pointer_move(&e),
                _ => {}
            }
        }
    }
}
