//! High level X11 wrapper.
//!
//! Only provides interfaces required by `nerdwm`.

pub mod context;
pub mod event;
pub mod input;
pub mod keysym;
pub mod window;

pub use xcb;
