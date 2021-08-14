//! Useful re-exports.

pub use crate::errors::*;
pub use log::{debug, error, info, trace, warn};

/// Get base directories based on the [`XDG specification`].
///
/// [`XDG specification`]: https://wiki.debian.org/XDGBaseDirectorySpecification
pub fn get_xdg_dirs() -> xdg::BaseDirectories {
    xdg::BaseDirectories::with_prefix("nerdwm").unwrap()
}
