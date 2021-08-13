//! Window manager implementation.

use std::sync::Arc;

use crate::events;
use crate::prelude::*;

pub mod desktop;
pub mod ewmh;
pub mod layout;

/// The window manager itself. This will keep track of
/// virtual desktops and handle events.
pub struct WindowManager {
    /// X server connection handle.
    conn: Arc<xcb::Connection>,
    /// Helper for EWMH and atoms.
    ewmh_mgr: Arc<ewmh::EWMHManager>,
    /// Helper for event processing.
    event_mgr: events::EventManager,
    /// Virtual desktops.
    desktops: Vec<desktop::Desktop>,
}

impl WindowManager {
    pub fn new() -> NerdResult<Self> {
        // Connect to the X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);
        let ewmh_mgr = Arc::new(ewmh::EWMHManager::new(conn.clone()));

        let mut wm = Self {
            conn: conn.clone(),
            ewmh_mgr: ewmh_mgr.clone(),
            event_mgr: events::EventManager::new(conn.clone()),
            // This is only here for testing
            desktops: vec![
                desktop::Desktop::new(
                    conn.clone(),
                    "main".to_owned(),
                    Box::new(layout::BlankLayout {}),
                    ewmh_mgr.clone(),
                ),
                desktop::Desktop::new(
                    conn.clone(),
                    "secondary".to_owned(),
                    Box::new(layout::BlankLayout {}),
                    ewmh_mgr.clone(),
                ),
            ],
        };

        wm.init()?;
        Ok(wm)
    }

    /// Setup event masks, required atoms, and load configurations.
    pub fn init(&mut self) -> NerdResult<()> {
        // Capture events on root. All events/requests for any
        // changes to its direct children can be captured and handled.
        xcb::change_window_attributes(
            &self.conn,
            self.get_root(),
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

        // Get existing windows
        xcb::grab_server_checked(&self.conn).request_check()?;

        // TODO

        // Grab bindings
        // -------------
        // These are temporary - will eventually be loaded from a
        // config file

        // We won't store a keysymbol struct ourselves
        let keysyms = events::keyconvert::KeySymbols::new(self.conn.clone());

        for k in [events::keysyms::XK_A, events::keysyms::XK_S] {
            xcb::grab_key_checked(
                &self.conn,
                true,
                self.get_root(),
                xcb::MOD_MASK_4 as u16,
                keysyms.get_keycode(k).next().unwrap(),
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            )
            .request_check()?;
        }

        xcb::ungrab_server_checked(&self.conn).request_check()?;
        self.conn.flush();

        info!("Initialized!");
        Ok(())
    }

    /// Get the default root window.
    fn get_root(&self) -> xcb::Window {
        self.conn.get_setup().roots().next().unwrap().root()
    }

    /// Runs the event loop.
    pub async fn run(&mut self) -> NerdResult<()> {
        while self.conn.has_error().is_ok() {
            self.conn.flush();

            match self.event_mgr.get_event()? {
                events::Event::WindowMapRequest(e) => {
                    self.desktops[0].focus(e.window())?;
                    self.ewmh_mgr.update_active_window(Some(e.window()))?;
                }
                events::Event::KeyPress(e) => {
                    // very basic test of desktop switching
                    if e.keysym() == events::keysyms::XK_A {
                        self.desktops[0].hide()?;
                    } else {
                        self.desktops[0].show()?;
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }
}
