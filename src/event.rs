//! X Event wrapper.

use x11_dl::xlib;

use serde::{Deserialize, Serialize};

/// X events.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Event {
    Unknown,
    WindowCreate(xlib::XCreateWindowEvent),
    WindowDestroy(xlib::XDestroyWindowEvent),
    WindowMapRequest(xlib::XMapRequestEvent),
    WindowUnmap(xlib::XUnmapEvent),
    WindowConfigureRequest(xlib::XConfigureRequestEvent),
}

impl From<xlib::XEvent> for Event {
    fn from(event: xlib::XEvent) -> Self {
        match event.get_type() {
            xlib::CreateNotify => Self::WindowCreate(unsafe { event.create_window }),
            xlib::DestroyNotify => Self::WindowDestroy(unsafe { event.destroy_window }),
            xlib::MapRequest => Self::WindowMapRequest(unsafe { event.map_request }),
            xlib::UnmapNotify => Self::WindowUnmap(unsafe { event.unmap }),
            xlib::ConfigureRequest => {
                Self::WindowConfigureRequest(unsafe { event.configure_request })
            }
            _ => Self::Unknown,
        }
    }
}

/// WM actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Action {
    None,
    WindowMove,
    WindowResize,
    WindowClose,
}

impl From<String> for Action {
    fn from(action: String) -> Self {
        match &action[..] {
            "WindowMove" => Self::WindowMove,
            "WindowResize" => Self::WindowResize,
            _ => Self::None,
        }
    }
}
