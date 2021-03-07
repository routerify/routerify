use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};

/// The error type used by the error handlers.
pub type HandleError = Box<dyn StdError + Send + Sync + 'static>;

/// The error type for simple string errors for compatibility with Routerify v1.
/// Can be used in return types of handlers and middleware.
pub struct Error {
    msg: String,
}

impl Error {
    /// Creates a new error instance with the specified message.
    pub fn new<M: Into<String>>(msg: M) -> Self {
        Error { msg: msg.into() }
    }

    /// Converts other error type to the `routerify::Error` type.
    pub fn wrap<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Error { msg: err.to_string() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "routerify::Error: {}", self.msg)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "routerify::Error: {}", self.msg)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}
