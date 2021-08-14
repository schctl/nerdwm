//! Error types.

/// Helper macro to implement conversions for native XCB error types.
macro_rules! xcb_error_impl {
    ($(
        $name:ident => $type:ty,
    )*) => {
        /// All XCB error types.
        pub enum XcbError {
            $($name($type),)*
        }

        $(
            impl From<$type> for XcbError {
                fn from(e: $type) -> Self {
                    Self::$name(e)
                }
            }

            impl From<$type> for Error {
                fn from(e: $type) -> Self {
                    Self::Xcb(XcbError::from(e))
                }
            }
        )*

        impl ::std::fmt::Debug for XcbError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(
                        Self::$name(_) => write!(f, stringify!($name)),
                    )*
                }
            }
        }
    };
}

xcb_error_impl! {
    Generic => xcb::GenericError,
    Atom => xcb::AtomError,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// A fatal error that results in a connection shutting down.
    Connection(xcb::ConnError),
    /// All XCB error types.
    Xcb(XcbError),
    /// Some resource was not found.
    NotFound(&'static str),
}

impl From<xcb::ConnError> for Error {
    fn from(e: xcb::ConnError) -> Self {
        Self::Connection(e)
    }
}

pub type NerdResult<T> = Result<T, Error>;
