pub use request_service::RequestService;
pub use router_service::RouterService;

pub(crate) type BoxedFutureResult<R, E> = Box<dyn std::future::Future<Output = Result<R, E>> + Send + 'static>;

mod request_service;
mod router_service;
