// This code derived from https://github.com/meh/rust-xcb-util
//
// Copyright (c) 2016 meh. <meh@schizofreni.co>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Keycode to Keysym conversion utilities.

use std::sync::Arc;

use libc::{c_void, free};
use xcb_util::ffi::keysyms::*;

pub struct KeySymbols {
    ptr: *mut xcb_key_symbols_t,
}

impl KeySymbols {
    pub fn new(c: &xcb::Connection) -> KeySymbols {
        unsafe {
            KeySymbols {
                ptr: xcb_key_symbols_alloc(c.get_raw_conn()),
            }
        }
    }

    pub fn get_keysym(&self, keycode: xcb::Keycode, col: i32) -> xcb::Keysym {
        unsafe { xcb_key_symbols_get_keysym(self.ptr, keycode, col) }
    }

    pub fn get_keycode(&self, keysym: xcb::Keysym) -> KeycodeIter {
        unsafe {
            KeycodeIter {
                ptr: xcb_key_symbols_get_keycode(self.ptr, keysym),
                index: 0,
            }
        }
    }

    pub fn press_lookup_keysym(&self, event: &xcb::KeyPressEvent, col: i32) -> xcb::Keysym {
        unsafe { xcb_key_press_lookup_keysym(self.ptr, event.ptr, col) }
    }

    pub fn release_lookup_keysym(&self, event: &xcb::KeyReleaseEvent, col: i32) -> xcb::Keysym {
        unsafe { xcb_key_release_lookup_keysym(self.ptr, event.ptr, col) }
    }

    pub fn refresh_keyboard_mapping(&self, event: &xcb::MappingNotifyEvent) -> i32 {
        unsafe { xcb_refresh_keyboard_mapping(self.ptr, event.ptr) }
    }
}

impl Drop for KeySymbols {
    fn drop(&mut self) {
        unsafe {
            xcb_key_symbols_free(self.ptr);
        }
    }
}

pub struct KeycodeIter {
    ptr: *mut xcb::Keycode,
    index: isize,
}

impl Drop for KeycodeIter {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr as *mut c_void);
        }
    }
}

impl Iterator for KeycodeIter {
    type Item = xcb::Keycode;

    fn next(&mut self) -> Option<xcb::Keycode> {
        unsafe {
            if self.ptr.is_null() {
                return None;
            }

            match *self.ptr.offset(self.index) {
                0 => None,

                keycode => {
                    self.index += 1;
                    Some(keycode)
                }
            }
        }
    }
}

pub fn is_keypad_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_keypad_key(keysym) != 0 }
}

pub fn is_private_keypad_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_private_keypad_key(keysym) != 0 }
}

pub fn is_cursor_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_cursor_key(keysym) != 0 }
}

pub fn is_pf_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_pf_key(keysym) != 0 }
}

pub fn is_function_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_function_key(keysym) != 0 }
}

pub fn is_misc_function_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_misc_function_key(keysym) != 0 }
}

pub fn is_modifier_key(keysym: xcb::Keysym) -> bool {
    unsafe { xcb_is_modifier_key(keysym) != 0 }
}
