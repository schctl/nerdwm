pub mod floating;

use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::client::ClientWindow;
use crate::display_context::DisplayContext;
use crate::window::Window;

/// Window border configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderConfig {
    /// Border width.
    pub width: u32,
    /// Border color for the focused window.
    pub color: u64,
    /// Border color for the unfocused window.
    pub unfocused_color: u64,
}

/// Layout configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
    pub gap_size: u32,
    pub border: BorderConfig,
}

/// Manage window position and sizes.
pub trait LayoutManager {
    /// Push a window to the stack.
    fn config(&self, windows: &Vec<ClientWindow>);
}
