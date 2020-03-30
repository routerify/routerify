pub use self::error::Error;
pub(crate) use self::error::{ErrorExt, ResultExt};
pub use self::helpers::{handle_request, handle_request_with_err};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::route::Route;
pub use self::router::{Router, RouterBuilder};

pub mod body;
mod error;
pub mod ext;
mod helpers;
mod middleware;
pub mod prelude;
mod route;
mod router;
pub mod types;
pub mod utility;

pub type Result<T> = std::result::Result<T, Error>;
