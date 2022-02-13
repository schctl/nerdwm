//! Helpers for accessing the [`EWMH`] protocol.
//!
//! [`EWMH`]: https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints

use std::sync::Arc;

use crate::atoms::AtomManager;
use crate::prelude::*;

// Atoms required by the EWMH protocol
define_string_consts! {
    pub protocols {
        _NET_SUPPORTED,
        _NET_WM_NAME,
        _NET_WM_PID,
        _NET_CLIENT_LIST,
        _NET_DESKTOP_NAMES,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_ACTIVE_WINDOW,
    }
}

/// Helper for setting EWMH hints.
///
/// Also provides general functions for managing properties / atoms.
pub struct EWMHManager {
    conn: Arc<xcb::Connection>,
    atoms: AtomManager,
}

impl EWMHManager {
    #[must_use]
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
        Self {
            conn: conn.clone(),
            atoms: AtomManager::new(conn),
        }
    }

    /// Get the default root window.
    fn get_root(&self) -> NerdResult<xcb::Window> {
        match self.conn.get_setup().roots().next() {
            Some(root) => Ok(root.root()),
            None => Err(Error::Static("root window not found")),
        }
    }

    /// Get the value of an atom.
    pub fn get_atom(&self, name: &'static str) -> NerdResult<xcb::Atom> {
        self.atoms.get(name)
    }

    /// Get supported protocols.
    pub fn get_net_supported(&self) -> NerdResult<Vec<xcb::Atom>> {
        Ok(vec![
            self.atoms.get(protocols::_NET_WM_NAME)?,
            self.atoms.get(protocols::_NET_WM_PID)?,
            self.atoms.get(protocols::_NET_CLIENT_LIST)?,
            self.atoms.get(protocols::_NET_DESKTOP_NAMES)?,
            self.atoms.get(protocols::_NET_NUMBER_OF_DESKTOPS)?,
            self.atoms.get(protocols::_NET_ACTIVE_WINDOW)?,
        ])
    }

    /// Change a property with type [`xcb::ATOM_ATOM`].
    pub fn set_property_atom(
        &self,
        window: xcb::Window,
        property: xcb::Atom,
        values: &[u32],
    ) -> NerdResult<()> {
        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            window,
            property,
            xcb::ATOM_ATOM,
            32,
            values,
        )
        .request_check()?;
        Ok(())
    }

    /// Change a property with type [`xcb::ATOM_CARDINAL`].
    pub fn set_property_cardinal(
        &self,
        window: xcb::Window,
        property: xcb::Atom,
        values: &[u32],
    ) -> NerdResult<()> {
        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            window,
            property,
            xcb::ATOM_CARDINAL,
            32,
            values,
        )
        .request_check()?;
        Ok(())
    }

    /// Change a property with type [`xcb::ATOM_STRING`].
    pub fn set_property_string(
        &self,
        window: xcb::Window,
        property: xcb::Atom,
        values: &[&str],
    ) -> NerdResult<()> {
        // :/
        let mut cstr_values: Vec<u8> = Vec::new();
        for val in values {
            cstr_values.extend(val.as_bytes());
            cstr_values.push(0);
        }

        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            window,
            property,
            self.get_atom("UTF8_STRING")?,
            8,
            &cstr_values[..],
        )
        .request_check()?;
        Ok(())
    }

    /// Change a property with type [`xcb::ATOM_WINDOW`].
    pub fn set_property_window(
        &self,
        window: xcb::Window,
        property: xcb::Atom,
        values: &[u32],
    ) -> NerdResult<()> {
        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            window,
            property,
            xcb::ATOM_WINDOW,
            32,
            values,
        )
        .request_check()?;
        Ok(())
    }

    /// Hint supported protocols.
    pub fn set_supported(&self) -> NerdResult<()> {
        self.set_property_atom(
            self.get_root()?,
            self.get_atom(protocols::_NET_SUPPORTED)?,
            &self.get_net_supported()?[..],
        )?;
        trace!("Successfully set supported hints");
        Ok(())
    }

    /// Set the `NET_WM_NAME` hint.
    pub fn set_name(&self, name: &str) -> NerdResult<()> {
        self.set_property_string(
            self.get_root()?,
            self.get_atom(protocols::_NET_WM_NAME)?,
            &[name],
        )?;
        trace!("Successfully set name hint");
        Ok(())
    }

    /// Set the `_NET_WM_PID` hint with the current process's ID.
    pub fn set_pid(&self) -> NerdResult<()> {
        self.set_property_cardinal(
            self.get_root()?,
            self.get_atom(protocols::_NET_WM_PID)?,
            &[std::process::id()],
        )?;
        trace!("Successfully set pid hint");
        Ok(())
    }

    /// Update desktop hints.
    ///
    /// These hints include:
    /// - `_NET_DESKTOP_NAMES`
    /// - `_NET_NUMBER_OF_DESKTOPS`
    pub fn update_desktops(&self, desktops: &[&str]) -> NerdResult<()> {
        // Number of desktops
        self.set_property_cardinal(
            self.get_root()?,
            self.get_atom(protocols::_NET_NUMBER_OF_DESKTOPS)?,
            &[desktops.len() as u32],
        )?;

        // Desktop names
        self.set_property_string(
            self.get_root()?,
            self.get_atom(protocols::_NET_DESKTOP_NAMES)?,
            desktops,
        )?;

        trace!("Successfully set desktop hints");
        Ok(())
    }

    /// Change the `_NET_ACTIVE_WINDOW` hint.
    pub fn update_active_window(&self, active: Option<xcb::Window>) -> NerdResult<()> {
        let win = if let Some(w) = active { w } else { xcb::NONE };

        self.set_property_window(
            self.get_root()?,
            self.get_atom(protocols::_NET_ACTIVE_WINDOW)?,
            &[win],
        )?;

        trace!("Successfully set active window");
        Ok(())
    }

    /// Update `_NET_CLIENT_LIST` with the list of clients being managed.
    pub fn update_client_list(&self, clients: &[xcb::Window]) -> NerdResult<()> {
        self.set_property_window(
            self.get_root()?,
            self.get_atom(protocols::_NET_CLIENT_LIST)?,
            clients,
        )?;

        trace!("Successfully updated client list");
        Ok(())
    }
}
