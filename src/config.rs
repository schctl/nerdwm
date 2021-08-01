//! Window manager configurations.

use log::*;
use x11_dl::keysym;
use x11_dl::xlib;

use serde::{Deserialize, Serialize};

use std::io::Read;
use std::path::Path;

use crate::event;
use crate::input;
use crate::layout;

/// Key + Modifiers for a window manager action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyBind {
    pub action: event::Action,
    pub bind: input::Key,
    pub modifiers: Vec<input::ModifierMask>,
}

impl KeyBind {
    pub fn get_mask(&self) -> u32 {
        let mut mask = 0;
        for modifier in &self.modifiers {
            mask |= u32::from(*modifier);
        }
        mask
    }
}

/// Button + Modifiers for a window manager action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MouseBind {
    pub action: event::Action,
    pub bind: input::Button,
    pub modifiers: Vec<input::ModifierMask>,
}

impl MouseBind {
    pub fn get_mask(&self) -> u32 {
        let mut mask = 0;
        for modifier in &self.modifiers {
            mask |= u32::from(*modifier);
        }
        mask
    }
}

/// Window Manager options.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub keybinds: Vec<KeyBind>,
    pub mousebinds: Vec<MouseBind>,
    pub layout: layout::LayoutConfig,
}

impl Config {
    pub fn new(path: &Path) -> Self {
        let mut config_file = std::fs::File::open(path).unwrap();
        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).unwrap();

        debug!("Parsed configuration file [{:#x?}]", path);

        serde_json::from_str(&config_string[..]).unwrap()
    }
}
