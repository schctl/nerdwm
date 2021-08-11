//! X Atom utilities.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::prelude::*;

/// Helper for keeping track of atoms.
pub struct AtomManager {
    conn: Arc<xcb::Connection>,
    atoms: Mutex<HashMap<&'static str, xcb::Atom>>,
}

impl AtomManager {
    #[must_use]
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
        Self {
            conn,
            atoms: Mutex::new(HashMap::new()),
        }
    }

    /// Retrieve an atom value.
    pub fn get(&self, name: &'static str) -> NerdResult<xcb::Atom> {
        let mut atoms_lock = self.atoms.lock().unwrap();

        if let Some(val) = atoms_lock.get(name) {
            Ok(*val)
        } else {
            let val = xcb::intern_atom(&self.conn, false, name)
                .get_reply()?
                .atom();
            atoms_lock.insert(name, val);
            Ok(val)
        }
    }
}

impl std::fmt::Debug for AtomManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} atoms stored", self.atoms.lock().unwrap().len())
    }
}

unsafe impl Send for AtomManager {}
unsafe impl Sync for AtomManager {}
