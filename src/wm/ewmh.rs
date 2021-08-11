//! [`EWMH`] utilities.
//!
//! [`EWMH`]: https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints

#![allow(dead_code)]

use std::sync::Arc;

use crate::atoms::AtomManager;
use crate::prelude::*;

// Atoms required by the EWMH protocol
define_properties_by_string! {
    pub supported {
        _NET_SUPPORTED,
        _NET_WM_NAME,
        _NET_WM_PID,
        _NET_CLIENT_LIST,
        _NET_DESKTOP_NAMES,
        _NET_NUMBER_OF_DESKTOPS,
    }
}

/// Helper for setting EWMH hints
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
    fn get_root(&self) -> xcb::Window {
        self.conn.get_setup().roots().next().unwrap().root()
    }

    /// Get an atom value.
    pub fn get_atom(&self, name: &'static str) -> NerdResult<xcb::Atom> {
        self.atoms.get(name)
    }

    /// Get supported protocols.
    pub fn get_net_supported(&self) -> NerdResult<Vec<xcb::Atom>> {
        Ok(vec![
            self.atoms.get(supported::_NET_WM_NAME)?,
            self.atoms.get(supported::_NET_WM_PID)?,
            self.atoms.get(supported::_NET_CLIENT_LIST)?,
            self.atoms.get(supported::_NET_DESKTOP_NAMES)?,
            self.atoms.get(supported::_NET_NUMBER_OF_DESKTOPS)?,
        ])
    }

    /// Change a property with type [`xcb::ATOM_ATOM`].
    fn set_property_atom(
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

    /// Change property with type [`xcb::ATOM_CARDINAL`].
    fn set_property_cardinal(
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

    /// Change property with type [`xcb::ATOM_STRING`].
    fn set_property_string(
        &self,
        window: xcb::Window,
        property: xcb::Atom,
        values: &[String],
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
            self.atoms.get("UTF8_STRING")?,
            8,
            &cstr_values[..],
        )
        .request_check()?;
        Ok(())
    }

    /// Hint supported protocols.
    pub fn set_net_supported(&self) -> NerdResult<()> {
        self.set_property_atom(
            self.get_root(),
            self.atoms.get(supported::_NET_SUPPORTED)?,
            &self.get_net_supported()?[..],
        )?;
        debug!("Successfully set supported hints");
        Ok(())
    }

    /// Set the `NET_WM_NAME` hint.
    pub fn set_name(&self, name: &str) -> NerdResult<()> {
        self.set_property_string(
            self.get_root(),
            self.atoms.get(supported::_NET_WM_NAME)?,
            &[name.to_owned()],
        )?;
        debug!("Successfully set name hint");
        Ok(())
    }

    /// Set the `_NET_WM_PID` hint with the current process's ID.
    pub fn set_pid(&self) -> NerdResult<()> {
        self.set_property_cardinal(
            self.get_root(),
            self.atoms.get(supported::_NET_WM_PID)?,
            &[std::process::id()],
        )?;
        debug!("Successfully set pid hint");
        Ok(())
    }

    /// Change desktop hints.
    pub fn update_desktops(&self, desktops: &[String]) -> NerdResult<()> {
        // Number of desktops
        self.set_property_cardinal(
            self.get_root(),
            self.atoms.get(supported::_NET_NUMBER_OF_DESKTOPS)?,
            &[desktops.len() as u32],
        )?;

        // Desktop names
        self.set_property_string(
            self.get_root(),
            self.atoms.get(supported::_NET_DESKTOP_NAMES)?,
            desktops,
        )?;

        debug!("Successfully set desktop hints");
        Ok(())
    }
}
