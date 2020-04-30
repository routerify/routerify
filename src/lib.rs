pub use self::error::Error;
pub(crate) use self::error::{ErrorExt, ResultExt};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::router::{Router, RouterBuilder};
pub use self::service::{RequestService, RouterService};
pub use self::types::PathParams;

mod constants;
mod error;
pub mod ext;
mod helpers;
mod middleware;
pub mod prelude;
mod regex_generator;
mod route;
mod router;
mod service;
mod types;

pub type Result<T> = std::result::Result<T, Error>;
