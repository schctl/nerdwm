//! X input mappings.

use x11_dl::keysym;
use x11_dl::xlib;

use serde::{Deserialize, Serialize};

/// Auto implement map.
macro_rules! key_map {
    (
        $name:ident {
            $(
                $field:ident => $sym:expr,
            )*
        }
    ) => {
        #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
        pub enum $name {
            $($field,)*
            Unknown
        }

        impl From<$name> for u32 {
            fn from(sym: $name) -> Self {
                match sym {
                    $(
                        $name::$field => $sym,
                    )*
                    _ => 0
                }
            }
        }
    };
}

key_map! {
    Key {
        A => keysym::XK_A,
        B => keysym::XK_A,
        C => keysym::XK_A,
        D => keysym::XK_A,
        E => keysym::XK_A,
        F => keysym::XK_A,
        G => keysym::XK_A,
        H => keysym::XK_A,
        I => keysym::XK_A,
        J => keysym::XK_A,
        K => keysym::XK_A,
        L => keysym::XK_A,
        M => keysym::XK_A,
        N => keysym::XK_A,
        O => keysym::XK_A,
        P => keysym::XK_A,
        Q => keysym::XK_A,
        R => keysym::XK_A,
        S => keysym::XK_A,
        T => keysym::XK_A,
        U => keysym::XK_A,
        V => keysym::XK_A,
        W => keysym::XK_A,
        X => keysym::XK_A,
        Y => keysym::XK_A,
        Z => keysym::XK_A,
        ArrowUp => keysym::XK_KP_Up,
        ArrowDown => keysym::XK_KP_Down,
        ArrowRight => keysym::XK_KP_Right,
        ArrowLeft => keysym::XK_KP_Left,
    }
}

key_map! {
    Button {
        Left => xlib::Button1,
        Middle => xlib::Button2,
        Right => xlib::Button3,
    }
}

key_map! {
    ModifierMask {
        Mod2 => xlib::Mod2Mask, // Alt
        Mod4 => xlib::Mod4Mask, // Super
        Shift => xlib::ShiftMask,
    }
}
