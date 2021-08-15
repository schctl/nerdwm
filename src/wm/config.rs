//! Window Manager configuration structures.

#![allow(unused)]

use serde::{Deserialize, Serialize};

use super::actions;
use crate::events::input;

/// Keyboard binding, consisting of a regular key press and an
/// optional modifier mask.
#[derive(Deserialize, Serialize)]
pub struct KeyBind {
    keysym: input::Key,
    modifiers: Vec<input::ModMask>,
}

impl KeyBind {
    #[must_use]
    pub fn new(keysym: input::Key, modifiers: Vec<input::ModMask>) -> Self {
        Self { keysym, modifiers }
    }

    /// Get the key symbol associated with this binding.
    pub fn get_keysym(&self) -> input::Key {
        self.keysym
    }

    /// Get the modifier mask for all modifiers associated with this binding.
    pub fn get_modifier_mask(&self) -> xcb::ModMask {
        let mut mask = 0;
        for modifier in &self.modifiers {
            mask |= *modifier as u32;
        }
        mask
    }
}

/// Mouse button binding, consisting of a regular mouse button press
/// and an optional modifier mask.
#[derive(Deserialize, Serialize)]
pub struct MouseBind {
    button: input::Button,
    modifiers: Vec<input::ModMask>,
}

impl MouseBind {
    #[must_use]
    pub fn new(button: input::Button, modifiers: Vec<input::ModMask>) -> Self {
        Self { button, modifiers }
    }

    /// Get the mouse button associated with this binding.
    pub fn get_button(&self) -> input::Button {
        self.button
    }

    /// Get the modifier mask for all modifiers associated with this binding.
    pub fn get_modifier_mask(&self) -> xcb::ModMask {
        let mut mask = 0;
        for modifier in &self.modifiers {
            mask |= *modifier as u32;
        }
        mask
    }
}

/// Configuration for bindings related to window manager actions.
#[derive(Deserialize, Serialize)]
pub struct ActionConfig {
    action: actions::ActionType,
    keybind: Option<KeyBind>,
    mousebind: Option<MouseBind>,
}

impl ActionConfig {
    #[must_use]
    pub fn new(
        action: actions::ActionType,
        keybind: Option<KeyBind>,
        mousebind: Option<MouseBind>,
    ) -> Self {
        Self {
            action,
            keybind,
            mousebind,
        }
    }

    pub fn get_type(&self) -> actions::ActionType {
        self.action
    }

    /// Get the key binding associated with this action.
    pub fn get_keybind(&self) -> &Option<KeyBind> {
        &self.keybind
    }

    /// Get the mouse binding associated with this action.
    pub fn get_mousebind(&self) -> &Option<MouseBind> {
        &self.mousebind
    }
}

/// Global window manager configurations.
#[derive(Deserialize, Serialize)]
pub struct Config {
    actions: Vec<ActionConfig>,
}

impl Config {
    #[must_use]
    pub fn from_str(config: &str) -> Self {
        // TODO: propagate `Result`.
        toml::from_str(config).unwrap()
    }

    pub fn get_actions(&self) -> &Vec<ActionConfig> {
        &self.actions
    }
}
