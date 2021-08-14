//! Window manager implementation.

use std::sync::Arc;

use crate::events;
use crate::prelude::*;

pub mod actions;
pub mod config;
pub mod desktop;
pub mod ewmh;
pub mod layout;

/// The "state" of the window manager. Processing of
/// events will depend on this.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    None,
    MovingWindow,
    ResizingWindow,
}

/// The window manager itself. This will keep track of virtual desktops and handle events.
pub struct WindowManager {
    /// X server connection handle.
    conn: Arc<xcb::Connection>,
    /// Helper for EWMH and atoms.
    ewmh_mgr: Arc<ewmh::EWMHManager>,
    /// Helper for event processing.
    event_mgr: events::EventManager,
    /// Virtual desktops.
    desktops: Vec<desktop::Desktop>,
    /// Global configurations.
    config: config::Config,
    /// Global mode.
    mode: Mode,
}

impl WindowManager {
    pub fn new() -> NerdResult<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);
        let ewmh_mgr = Arc::new(ewmh::EWMHManager::new(conn.clone()));

        // TODO: accept absolute path as argument to read from, and generate non-existent configs.
        let config = {
            let config_str = include_str!("../../assets/config.toml");
            config::Config::from_str(config_str)
        };

        let mut wm = Self {
            conn: conn.clone(),
            ewmh_mgr: ewmh_mgr.clone(),
            event_mgr: events::EventManager::new(conn.clone()),
            config,
            mode: Mode::None,
            // TODO: read from config
            desktops: vec![desktop::Desktop::new(
                conn,
                "main".to_owned(),
                Box::new(layout::BlankLayout {}),
                ewmh_mgr,
            )],
        };

        wm.init()?;
        Ok(wm)
    }

    /// Setup event masks, required atoms, and load configurations.
    pub fn init(&mut self) -> NerdResult<()> {
        let root = self.get_root()?;

        // Capture events on root. All events/requests for any
        // changes to its direct children can be captured and handled.
        xcb::change_window_attributes(
            &self.conn,
            root,
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
            )],
        )
        .request_check()?;

        // Setup EWMH hints
        // ----------------
        self.ewmh_mgr.set_supported()?;
        self.ewmh_mgr.set_pid()?;
        self.ewmh_mgr.set_name("nerdwm")?;
        self.ewmh_mgr.update_active_window(None)?;
        self.ewmh_mgr.update_desktops(
            &self
                .desktops
                .iter()
                .map(|d| &d.get_name()[..])
                .collect::<Vec<&str>>()[..],
        )?;

        self.conn.flush();

        // TODO: Get existing windows
        xcb::grab_server_checked(&self.conn).request_check()?;

        // Grab bindings
        for action in self.config.get_actions() {
            if let Some(k) = action.get_keybind() {
                if let Some(keycode) = self
                    .event_mgr
                    .get_keysyms()
                    .get_keycode(k.get_keysym() as u32)
                    .next()
                {
                    xcb::grab_key_checked(
                        &self.conn,
                        true, // owner events
                        root,
                        k.get_modifier_mask() as u16,
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8, // pointer mode
                        xcb::GRAB_MODE_ASYNC as u8, // keyboard mode
                    )
                    .request_check()?;
                } else {
                    error!("Unable to get keycode for sym {:?}", k.get_keysym());
                }
            }

            if let Some(b) = action.get_mousebind() {
                xcb::grab_button_checked(
                    &self.conn,
                    false, // owner events
                    root,
                    (xcb::EVENT_MASK_BUTTON_PRESS
                        | xcb::EVENT_MASK_BUTTON_RELEASE
                        | xcb::EVENT_MASK_POINTER_MOTION) as u16, // event mask
                    xcb::GRAB_MODE_ASYNC as u8, // pointer mode
                    xcb::GRAB_MODE_ASYNC as u8, // keyboard mode
                    0,                          // confine to window
                    0,                          // cursor
                    b.get_button() as u8,
                    b.get_modifier_mask() as u16,
                )
                .request_check()?;
            }
        }

        xcb::ungrab_server_checked(&self.conn).request_check()?;
        self.conn.flush();

        info!("Initialized!");
        Ok(())
    }

    /// Get the default root window.
    fn get_root(&self) -> NerdResult<xcb::Window> {
        match self.conn.get_setup().roots().next() {
            Some(root) => Ok(root.root()),
            None => Err(Error::NotFound("root window")),
        }
    }

    /// Tries to resolve an event into an action
    fn event_to_action(&mut self, event: events::Event) -> Option<actions::Action> {
        // TODO: match mode inside event matches, so other events can be handled
        // without making too much of a mess.
        match &self.mode {
            Mode::None => match &event {
                events::Event::ButtonPress(e) => {
                    for action in self.config.get_actions() {
                        if let Some(b) = action.get_mousebind() {
                            if b.get_modifier_mask() == e.state() as u32
                                && b.get_button() as u8 == e.detail()
                            {
                                // TODO: We'll need to match against `action.get_type()` here
                                // to determine if the mode is actually `MovingWindow`.
                                // But this is fine for now.
                                self.mode = Mode::MovingWindow;
                                return Some(actions::Action::new(action.get_type(), event));
                            }
                        }
                    }
                }
                events::Event::ButtonRelease(e) => {
                    for action in self.config.get_actions() {
                        if let Some(b) = action.get_mousebind() {
                            if b.get_modifier_mask() == e.state() as u32
                                && b.get_button() as u8 == e.detail()
                            {
                                self.mode = Mode::None;
                                return Some(actions::Action::new(action.get_type(), event));
                            }
                        }
                    }
                }
                _ => {}
            },
            Mode::MovingWindow => match &event {
                events::Event::PointerMotion(_) => {
                    return Some(actions::Action::new(actions::ActionType::WindowMove, event));
                }
                _ => {}
            },
            _ => {}
        }

        None
    }

    /// Runs the event loop.
    pub async fn run(&mut self) -> NerdResult<()> {
        while self.conn.has_error().is_ok() {
            self.conn.flush();

            let event = self.event_mgr.get_event()?;

            match event {
                // This will be fixed with TODO on line 165
                events::Event::WindowMapRequest(e) => {
                    self.desktops[0].focus(e.window())?;
                }
                _ => {
                    if let Some(action) = self.event_to_action(event) {
                        self.desktops[0].do_action(action)?;
                    }
                }
            };
        }

        Ok(())
    }
}
