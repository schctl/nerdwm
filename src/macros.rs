//! Useful macros.
//!
//! These macros are defined here since they could be useful in multiple places.
//! Not all macros in this crate will be defined here as other modules will define
//! macros within themselves for use locally.

/// Define constants with names provided, and values equal to their names, in a module.
///
/// # Examples
/// ```
/// define_string_consts! {
///     pub foo {
///         BAR,
///         BAZ,
///     }
/// }
///
/// assert_eq!(foo::BAR, "BAR");
/// assert_eq!(foo::BAZ, "BAZ");
/// ```
macro_rules! define_string_consts {
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
