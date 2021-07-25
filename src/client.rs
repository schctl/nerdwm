//! Client utilities.

use crate::window;

/// Client window and decorations.
pub struct ClientWindow {
    pub internal: window::Window,
    pub frame: window::Window,
}
