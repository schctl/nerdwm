//! X Event wrapper.

/// Events returned by the X server connection.
#[non_exhaustive]
pub enum Event {
    Unknown,

    WindowCreate(xcb::CreateNotifyEvent),
    WindowDestroy(xcb::DestroyNotifyEvent),
    WindowMapRequest(xcb::MapRequestEvent),
    WindowUnmap(xcb::UnmapNotifyEvent),
    WindowConfigureRequest(xcb::ConfigureRequestEvent),

    ButtonPress(xcb::ButtonPressEvent),
    ButtonRelease(xcb::ButtonReleaseEvent),
    KeyPress(xcb::KeyPressEvent),
    KeyRelease(xcb::KeyReleaseEvent),
    PointerMotion(xcb::MotionNotifyEvent),
}

impl From<xcb::GenericEvent> for Event {
    fn from(event: xcb::GenericEvent) -> Self {
        match event.response_type() {
            xcb::CREATE_NOTIFY => Self::WindowCreate(unsafe { std::mem::transmute(event) }),
            xcb::DESTROY_NOTIFY => Self::WindowDestroy(unsafe { std::mem::transmute(event) }),
            xcb::MAP_REQUEST => Self::WindowMapRequest(unsafe { std::mem::transmute(event) }),
            xcb::UNMAP_NOTIFY => Self::WindowUnmap(unsafe { std::mem::transmute(event) }),
            xcb::CONFIGURE_REQUEST => {
                Self::WindowConfigureRequest(unsafe { std::mem::transmute(event) })
            }
            xcb::BUTTON_PRESS => Self::ButtonPress(unsafe { std::mem::transmute(event) }),
            xcb::BUTTON_RELEASE => Self::ButtonRelease(unsafe { std::mem::transmute(event) }),
            xcb::KEY_PRESS => Self::KeyPress(unsafe { std::mem::transmute(event) }),
            xcb::KEY_RELEASE => Self::KeyRelease(unsafe { std::mem::transmute(event) }),
            xcb::MOTION_NOTIFY => Self::PointerMotion(unsafe { std::mem::transmute(event) }),
            _ => Self::Unknown,
        }
    }
}
