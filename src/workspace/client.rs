//! Window Manager client utilities.

use nerdwm_x11::context::DisplayContext;
use nerdwm_x11::window::Window;

use super::layout::BorderConfig;

/// Client window and decorations.
#[derive(Debug, Clone, Copy)]
pub struct ClientWindow {
    /// The actual window.
    pub internal: Window,
    /// Parent window containing decorations.
    pub frame: Window,
}

impl ClientWindow {
    /// Create a frame for an already existing X window.
    pub fn from_window(context: &DisplayContext, window: Window, border: &BorderConfig) -> Self {
        let geometry = window.get_geometry(context).get_reply().unwrap();

        let frame = Window::create(
            context,
            &context.get_default_root(),
            geometry.x(),
            geometry.y(),
            geometry.width(),
            geometry.height(),
            border.width as u16,
        );

        frame.set_save_set(context, true);
        window.reparent(context, &frame);

        Self {
            internal: window,
            frame,
        }
    }

    /// Destroy the window frame, returning the internal window (which may or may not exist).
    pub fn destroy(self, context: &DisplayContext, reparent: bool) -> Window {
        if reparent {
            self.internal.reparent(context, &context.get_default_root());
        }
        self.frame.unmap(context);
        self.frame.set_save_set(context, false);
        self.frame.destroy(context);

        self.internal
    }
}
