pub use self::error::Error;
pub(crate) use self::error::{ErrorExt, ResultExt};
pub use self::helpers::{handle_request, handle_request_err};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::route::Route;
pub use self::router::{Router, RouterBuilder};

mod error;
pub mod ext;
pub mod handlers;
mod helpers;
mod middleware;
pub mod prelude;
mod route;
mod router;
pub mod types;

pub type Result<T> = std::result::Result<T, Error>;
