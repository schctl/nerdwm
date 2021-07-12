use log::{debug, info};
use x11_dl::xlib;

use crate::display_manager::DisplayManager;

pub type WindowProperties = xlib::XWindowAttributes;

pub struct Frame {
    pub id: u64,
    pub properties: WindowProperties,
}

impl Frame {
    pub fn new(context: &DisplayManager, id: u64) -> Self {
        Self {
            id,
            properties: context.get_window_properties(id),
        }
    }
}

pub struct Window {
    pub id: u64,
    pub properties: WindowProperties,
    pub frame: Frame,
}

impl Window {
    /// Create a new window and frame the window.
    pub fn new(context: &DisplayManager, id: u64) -> Self {
        let properties = context.get_window_properties(id);
        let frame = context.create_window(properties, 0x111111, 5, 0xffffff);
        let frame = Frame::new(context, frame);

        // Input events on frame
        context.set_event_mask(
            frame.id,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        );

        // Add frame to save set
        // The save-set of a client is a list of other clients' windows that, if they are
        // inferiors of one of the client's windows at connection close, should not be
        // destroyed and should be remapped if they are unmapped.
        // https://tronche.com/gui/x/xlib/window-and-session-manager/controlling-window-lifetime.html
        context.add_to_save_set(frame.id);

        // Reparent window to frame
        context.reparent_window(id, frame.id);

        // Map the frame
        context.map_window(frame.id);

        debug!("Created window frame.");

        // Map the window
        context.map_window(id);

        info!("Showing window {}", id);

        Self {
            id,
            properties,
            frame,
        }
    }

    /// Destroy window frame and consume self.
    pub fn destroy(self, context: &DisplayManager) {
        // Unmap the window frame first
        context.unmap_window(self.frame.id);

        // Reparent the window to the root
        context.reparent_window(self.id, context.root);

        // Remove frame from save set
        context.remove_from_save_set(self.frame.id);

        // Destroy frame
        context.destroy_window(self.frame.id);
    }
}
