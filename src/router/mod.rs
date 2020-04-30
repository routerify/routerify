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

pub struct Router<B, E> {
    pub(crate) pre_middlewares: Vec<PreMiddleware<B, E>>,
    pub(crate) routes: Vec<Route<B, E>>,
    pub(crate) post_middlewares: Vec<PostMiddleware<B, E>>,
    // This handler should be added only on root Router.
    // Any error handler attached to scoped router will be ignored.
    pub(crate) err_handler: Option<ErrHandler<B>>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Router<B, E> {
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
