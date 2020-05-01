use crate::middleware::{PostMiddleware, PreMiddleware};
use crate::prelude::*;
use crate::route::Route;
use hyper::{body::HttpBody, Request, Response};
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

pub use self::builder::Builder as RouterBuilder;

mod builder;

pub(crate) type ErrHandler<B> = Box<dyn FnMut(crate::Error) -> ErrHandlerReturn<B> + Send + Sync + 'static>;
pub(crate) type ErrHandlerReturn<B> = Box<dyn Future<Output = Response<B>> + Send + 'static>;

/// Represents a modular, lightweight and mountable router type.
///
/// A router consists of some routes, some pre-middlewares and some post-middlewares. The `Router<B, E>` type accepts two tye parameters: `B` and `E`.
///
/// The `B` represents the request body type which will be used across route handlers and this body type must implement
/// the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.13.5/hyper/body/struct.Body.html)
/// type as `Router<hyper::Body, hyper::Error>`.
///
/// The `E` represents any error type which will be usesd across the route handlers. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```
/// use routerify::Router;
/// use hyper::{Response, Request, Body};
///
/// // A handler for "/about" page.
/// // We will use hyper::Body as request body type and hyper::Error as error type.
/// async fn about_handler(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
///     Ok(Response::new(Body::from("About page")))
/// }
///
/// # fn run() -> Router<Body, hyper::Error> {
/// // Create a router with hyper::Body as request body type and hyper::Error as error type.
/// let router: Router<Body, hyper::Error> = Router::builder()
///     .get("/about", about_handler)
///     .build()
///     .unwrap();
/// # router
/// # }
/// # run();
/// ```
///
/// A `Router` can be created using the `Router::builder()` method.
pub struct Router<B, E> {
    pub(crate) pre_middlewares: Vec<PreMiddleware<B, E>>,
    pub(crate) routes: Vec<Route<B, E>>,
    pub(crate) post_middlewares: Vec<PostMiddleware<B, E>>,
    // This handler should be added only on root Router.
    // Any error handler attached to scoped router will be ignored.
    pub(crate) err_handler: Option<ErrHandler<B>>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Router<B, E> {
    /// Return a [RouterBuilder](./struct.RouterBuilder.html) instance to build a `Router`.
    pub fn builder() -> RouterBuilder<B, E> {
        builder::Builder::new()
    }

    pub(crate) async fn process(&mut self, req: Request<B>) -> crate::Result<Response<B>> {
        let target_path = req.uri().path().to_string();

        let Router {
            ref mut pre_middlewares,
            ref mut routes,
            ref mut post_middlewares,
            ..
        } = self;

        let mut transformed_req = req;
        for pre_middleware in pre_middlewares.iter_mut() {
            if pre_middleware.is_match(target_path.as_str()) {
                transformed_req = pre_middleware
                    .process(transformed_req)
                    .await
                    .context("One of the pre middlewares couldn't process the request")?;
            }
        }

        let mut resp: Option<Response<B>> = None;
        for route in routes.iter_mut() {
            if route.is_match(target_path.as_str(), transformed_req.method()) {
                let route_resp_res = route
                    .process(target_path.as_str(), transformed_req)
                    .await
                    .context("One of the routes couldn't process the request");

                let route_resp = match route_resp_res {
                    Ok(route_resp) => route_resp,
                    Err(err) => {
                        if let Some(ref mut err_handler) = self.err_handler {
                            Pin::from(err_handler(err)).await
                        } else {
                            return crate::Result::Err(err);
                        }
                    }
                };

                resp = Some(route_resp);
                break;
            }
        }

        if let None = resp {
            return Err(crate::Error::new("No handlers added to handle non-existent routes. Tips: Please add an '.any' route at the bottom to handle any routes."));
        }

        let mut transformed_res = resp.unwrap();
        for post_middleware in post_middlewares.iter_mut() {
            if post_middleware.is_match(target_path.as_str()) {
                transformed_res = post_middleware
                    .process(transformed_res)
                    .await
                    .context("One of the post middlewares couldn't process the response")?;
            }
        }

        Ok(transformed_res)
    }
}

impl<B, E> Debug for Router<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ Pre-Middlewares: {:?}, Routes: {:?}, Post-Middlewares: {:?}, ErrHandler: {:?} }}",
            self.pre_middlewares,
            self.routes,
            self.post_middlewares,
            self.err_handler.is_some()
        )
    }
}
