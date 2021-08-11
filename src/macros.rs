//! Helpful macros.
//!
//! These macros are defined here since they
//! could be useful in multiple places. Other
//! modules will define macros within themselves
//! for use locally.

/// Define constants with names provided and values being their
/// names, in a module.
macro_rules! define_properties_by_string {
    (
        $mod:ident {
            $($name:ident,)*
        }
    ) => {
        mod $mod {
            $(pub const $name: &str = stringify!($name);)*
        }
    };

    (
        pub $mod:ident {
            $($name:ident,)*
        }
    ) => {
        pub mod $mod {
            $(pub const $name: &str = stringify!($name);)*
        }
    };
}
