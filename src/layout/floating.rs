//! Floating window layout implementation.
//! Does basically nothing.

use super::*;
use crate::workspace::client::ClientWindow;

/// Floating window layout implementation.
pub struct FloatingLayoutManager {}

impl LayoutManager for FloatingLayoutManager {
    fn config(&self, _: &[ClientWindow]) {}
}
