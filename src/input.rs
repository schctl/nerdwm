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
        B => keysym::XK_B,
        C => keysym::XK_C,
        D => keysym::XK_D,
        E => keysym::XK_E,
        F => keysym::XK_F,
        G => keysym::XK_G,
        H => keysym::XK_H,
        I => keysym::XK_H,
        J => keysym::XK_J,
        K => keysym::XK_K,
        L => keysym::XK_L,
        M => keysym::XK_M,
        N => keysym::XK_N,
        O => keysym::XK_O,
        P => keysym::XK_P,
        Q => keysym::XK_Q,
        R => keysym::XK_R,
        S => keysym::XK_S,
        T => keysym::XK_T,
        U => keysym::XK_U,
        V => keysym::XK_V,
        W => keysym::XK_W,
        X => keysym::XK_X,
        Y => keysym::XK_Y,
        Z => keysym::XK_Z,
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
        Mod1 => xlib::Mod1Mask,  // Alt
        Mod2 => xlib::Mod2Mask,  // Num Lock
        Mod3 => xlib::Mod3Mask,  // Scroll Lock
        Mod4 => xlib::Mod4Mask,  // Super
        Shift => xlib::ShiftMask,
        CapsLock => xlib::LockMask,
        Control => xlib::ControlMask,
    }
}
