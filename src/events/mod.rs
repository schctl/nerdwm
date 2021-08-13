//! X event utilities.

use crate::prelude::*;
use std::sync::Arc;

pub mod keyconvert;
pub mod keysyms;

/// Wrapper containing an [`xcb::KeyPressEvent`] and it's corresponding
/// keysym for a specific connection.
pub struct KeyPressEvent {
    pub base: xcb::KeyPressEvent,
    keysym: xcb::Keysym,
}

impl KeyPressEvent {
    pub fn new(base: xcb::KeyPressEvent, keysym: xcb::Keysym) -> Self {
        Self { base, keysym }
    }

    #[allow(unused)]
    pub fn keysym(&self) -> xcb::Keysym {
        self.keysym
    }
}

/// Wrapper containing an [`xcb::KeyReleaseEvent`] and it's corresponding
/// keysym for a specific connection.
pub struct KeyReleaseEvent {
    pub base: xcb::KeyReleaseEvent,
    keysym: xcb::Keysym,
}

impl KeyReleaseEvent {
    pub fn new(base: xcb::KeyReleaseEvent, keysym: xcb::Keysym) -> Self {
        Self { base, keysym }
    }

    #[allow(unused)]
    pub fn keysym(&self) -> xcb::Keysym {
        self.keysym
    }
}

/// (Incomplete) list of events propagated by the X server.
#[non_exhaustive]
pub enum Event {
    Unknown,
    ClientMessage(xcb::ClientMessageEvent),

    WindowCreate(xcb::CreateNotifyEvent),
    WindowDestroy(xcb::DestroyNotifyEvent),
    WindowMapRequest(xcb::MapRequestEvent),
    WindowUnmap(xcb::UnmapNotifyEvent),
    WindowConfigureRequest(xcb::ConfigureRequestEvent),

    ButtonPress(xcb::ButtonPressEvent),
    ButtonRelease(xcb::ButtonReleaseEvent),
    PointerMotion(xcb::MotionNotifyEvent),

    KeyPress(KeyPressEvent),
    KeyRelease(KeyReleaseEvent),
}

/// Helper for converting received events into native types.
pub struct EventManager {
    conn: Arc<xcb::Connection>,
    keysyms: keyconvert::KeySymbols,
}

impl EventManager {
    #[must_use]
    pub fn new(conn: Arc<xcb::Connection>) -> Self {
        Self {
            conn: conn.clone(),
            keysyms: keyconvert::KeySymbols::new(conn),
        }
    }

    /// Wait for an event from the connection.
    pub fn get_event(&self) -> NerdResult<Event> {
        let event = match self.conn.wait_for_event() {
            Some(e) => e,
            None => return Err(Error::NotFound("event IO")),
        };

        Ok(match event.response_type() {
            xcb::CLIENT_MESSAGE => Event::ClientMessage(unsafe { std::mem::transmute(event) }),
            xcb::CREATE_NOTIFY => Event::WindowCreate(unsafe { std::mem::transmute(event) }),
            xcb::DESTROY_NOTIFY => Event::WindowDestroy(unsafe { std::mem::transmute(event) }),
            xcb::MAP_REQUEST => Event::WindowMapRequest(unsafe { std::mem::transmute(event) }),
            xcb::UNMAP_NOTIFY => Event::WindowUnmap(unsafe { std::mem::transmute(event) }),
            xcb::CONFIGURE_REQUEST => {
                Event::WindowConfigureRequest(unsafe { std::mem::transmute(event) })
            }
            xcb::BUTTON_PRESS => Event::ButtonPress(unsafe { std::mem::transmute(event) }),
            xcb::BUTTON_RELEASE => Event::ButtonRelease(unsafe { std::mem::transmute(event) }),
            xcb::KEY_PRESS => {
                let event: xcb::KeyPressEvent = unsafe { std::mem::transmute(event) };
                let keysym = self
                    .keysyms
                    .press_lookup_keysym(&event, 1);
                Event::KeyPress(KeyPressEvent::new(event, keysym))
            }
            xcb::KEY_RELEASE => {
                let event: xcb::KeyReleaseEvent = unsafe { std::mem::transmute(event) };
                let keysym = self
                    .keysyms
                    .press_lookup_keysym(&event, 1);
                Event::KeyRelease(KeyReleaseEvent::new(event, keysym))
            }
            xcb::MOTION_NOTIFY => Event::PointerMotion(unsafe { std::mem::transmute(event) }),
            _ => Event::Unknown,
        })
    }
}
