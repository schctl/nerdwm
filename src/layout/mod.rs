//! Tools for window layout management.
//!
//! Provides tools for resizing/moving windows
//! based on an implemented algorithm.

mod floating;
pub use floating::*;

use serde::{Deserialize, Serialize};

use crate::workspace::client::ClientWindow;

/// Window border configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorderConfig {
    /// Border width.
    pub width: u32,
    /// Border color for the focused window.
    pub color: u64,
}

/// Layout configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
    /// Size of gaps between windows.
    pub gap_size: u32,
    /// Border configuration for frames.
    pub border: BorderConfig,
    /// Border configuration for non-focused window frames.
    pub border_unfocused: BorderConfig,
}

/// Manage window position and sizes.
pub trait LayoutManager {
    /// Push a window to the stack.
    fn config(&self, windows: &[ClientWindow]);
}
