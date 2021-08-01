use std::rc::Rc;

use log::*;
use x11_dl::xlib;

use super::*;
use crate::client::ClientWindow;
use crate::display_context::DisplayContext;
use crate::window::Window;

/// Floating window layout implementation.
pub struct FloatingLayoutManager {}

impl LayoutManager for FloatingLayoutManager {
    fn config(&self, _: &Vec<ClientWindow>) {}
}
