//! Window Manager client utilities.

use x11_dl::xlib;

use crate::context::DisplayContext;
use crate::layout::BorderConfig;
use crate::window::Window;

/// Client window and decorations.
#[derive(Debug, Clone, Copy)]
pub struct ClientWindow {
    /// The actual window.
    pub internal: Window,
    /// Parent window containing decorations.
    pub frame: Window,
}

impl ClientWindow {
    pub fn from_window(context: &DisplayContext, window: Window, border: &BorderConfig) -> Self {
        let properties = window.get_properties(&context);

        let frame = Window::create(
            &context,
            &context.get_default_root(),
            properties.x,
            properties.y,
            properties.width as u32,
            properties.height as u32,
            border.width,
            border.color,
            0x0011_1111,
        );

        frame.set_event_mask(
            &context,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        );
        frame.set_save_set(&context, true);
        window.reparent(&context, &frame);

        Self {
            internal: window,
            frame,
        }
    }

    /// Destroy the window frame.
    pub fn destroy(self, context: &DisplayContext, reparent: bool) {
        if reparent {
            self.internal
                .reparent(&context, &context.get_default_root());
        }
        self.frame.unmap(&context);
        self.frame.set_save_set(&context, false);
        self.frame.destroy(&context);
    }
}
