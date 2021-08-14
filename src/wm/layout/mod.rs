//! Provides algorithms for configuring window geometry.

use crate::prelude::*;

pub trait Layout {
    fn configure(&self, clients: &[xcb::Window]) -> NerdResult<()>;
}

/// A layout that does nothing.
pub struct BlankLayout {}

impl Layout for BlankLayout {
    fn configure(&self, _: &[xcb::Window]) -> NerdResult<()> {
        Ok(())
    }
}
