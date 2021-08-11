//! Window manager implementation.

use std::sync::Arc;

use crate::events;
use crate::prelude::*;

pub mod ewmh;

pub struct WindowManager {
    conn: Arc<xcb::Connection>,
    /// Helpers for creating EWMH hints.
    /// This will also store all our atoms.
    ewmh_mgr: ewmh::EWMHManager,
    event_mgr: events::EventManager,
}

impl WindowManager {
    pub fn new() -> NerdResult<Self> {
        // Connect to X server
        let conn = Arc::new(xcb::Connection::connect(None)?.0);

        let mut wm = Self {
            conn: conn.clone(),
            ewmh_mgr: ewmh::EWMHManager::new(conn.clone()),
            event_mgr: events::EventManager::new(conn),
        };

        wm.init()?;
        Ok(wm)
    }

    /// Setup event masks, required atoms, and load configuration.
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
        self.ewmh_mgr.set_net_supported()?;
        self.ewmh_mgr.set_pid()?;
        self.ewmh_mgr.set_name("nerdwm")?;
        // testing for now
        self.ewmh_mgr
            .update_desktops(&["main".to_owned(), "secondary".to_owned()])?;

        // Clear client list
        xcb::delete_property(
            &self.conn,
            self.get_root(),
            self.ewmh_mgr.get_atom(ewmh::supported::_NET_CLIENT_LIST)?,
        )
        .request_check()?;

        // Flush requests
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
                    xcb::map_window(&self.conn, e.window());
                }
                _ => {}
            };
        }

        Ok(())
    }
}
