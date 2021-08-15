//! Window manager actions.
//!
//! An action can be anything from re-configuring a window's
//! geometry, to closing or restarting the window manager itself.

use crate::events;
use serde::{Deserialize, Serialize};

/// Represents all actions the window manager can perform.
/// Actions are how the window manager and desktops interpret
/// standard events.
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ActionType {
    FloatingWindowMove,
    FloatingWindowResize,
    /// For internal use.
    WindowFocus,
    WindowManagerQuit,
    WindowManagerRestart,
}

/// Represents an action corresponding to an event. This is what will
/// usually be passed around as the event holds information such as
/// the window to perform the action on. Actions not associated to
/// any other information will be simply stored as an [`ActionType`].
/// How actions are processed are implemented by [`super::desktop::Desktop`].
#[derive(Debug)]
pub struct Action {
    action: ActionType,
    event: events::Event,
}

impl Action {
    #[must_use]
    pub fn new(action: ActionType, event: events::Event) -> Self {
        Self { action, event }
    }

    /// Get the type of action to perform.
    pub fn get_type(&self) -> ActionType {
        self.action
    }

    /// Get the event that is associated to this action.
    pub fn get_event(&self) -> &events::Event {
        &self.event
    }
}
