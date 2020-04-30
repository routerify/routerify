use crate::middleware::{Middleware, PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::router::ErrHandler;
use crate::router::Router;
use hyper::{body::HttpBody, Method, Request, Response};
use std::future::Future;

struct BuilderInner<B, E> {
    pre_middlewares: Vec<PreMiddleware<B, E>>,
    routes: Vec<Route<B, E>>,
    post_middlewares: Vec<PostMiddleware<B, E>>,
    err_handler: Option<ErrHandler<B>>,
}

pub struct Builder<B, E> {
    inner: crate::Result<BuilderInner<B, E>>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Builder<B, E> {
    pub fn new() -> Builder<B, E> {
        Builder::default()
    }

    pub fn build(self) -> crate::Result<Router<B, E>> {
        self.inner.map(|inner| Router {
            pre_middlewares: inner.pre_middlewares,
            routes: inner.routes,
            post_middlewares: inner.post_middlewares,
            err_handler: inner.err_handler,
        })
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(BuilderInner<B, E>) -> crate::Result<BuilderInner<B, E>>,
    {
        Builder {
            inner: self.inner.and_then(func),
        }
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Builder<B, E> {
    pub fn get<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::GET], handler)
    }

    pub fn get_or_head<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::GET, Method::HEAD], handler)
    }

    pub fn post<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::POST], handler)
    }

    pub fn put<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::PUT], handler)
    }

    pub fn delete<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::DELETE], handler)
    }

    pub fn head<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::HEAD], handler)
    }

    pub fn trace<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::TRACE], handler)
    }

    pub fn connect<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::CONNECT], handler)
    }

    pub fn patch<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::PATCH], handler)
    }

    pub fn options<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::OPTIONS], handler)
    }

    pub fn any<H, R>(self, handler: H) -> Self
    where
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(
            "/*",
            vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::CONNECT,
                Method::HEAD,
                Method::OPTIONS,
                Method::TRACE,
            ],
            handler,
        )
    }

    pub fn add<P, H, R>(self, path: P, methods: Vec<Method>, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.and_then(move |mut inner| {
            let route = Route::new(path, methods, handler)?;
            inner.routes.push(route);
            crate::Result::Ok(inner)
        })
    }

    pub fn scope<P>(self, path: P, mut router: Router<B, E>) -> Self
    where
        P: Into<String>,
    {
        let mut path = path.into();

        if path.ends_with("/") {
            path = (&path[..path.len() - 1]).to_string();
        }

        let mut builder = self;

        for pre_middleware in router.pre_middlewares.iter_mut() {
            let new_pre_middleware = PreMiddleware::new_with_boxed_handler(
                format!("{}{}", path.as_str(), pre_middleware.path.as_str()),
                pre_middleware
                    .handler
                    .take()
                    .expect("No handler found in one of the pre-middlewares"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.pre_middlewares.push(new_pre_middleware?);
                crate::Result::Ok(inner)
            });
        }

        for route in router.routes.iter_mut() {
            let new_route = Route::new_with_boxed_handler(
                format!("{}{}", path.as_str(), route.path.as_str()),
                route.methods.clone(),
                route.handler.take().expect("No handler found in one of the routes"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.routes.push(new_route?);
                crate::Result::Ok(inner)
            });
        }

        for post_middleware in router.post_middlewares.iter_mut() {
            let new_post_middleware = PostMiddleware::new_with_boxed_handler(
                format!("{}{}", path.as_str(), post_middleware.path.as_str()),
                post_middleware
                    .handler
                    .take()
                    .expect("No handler found in one of the post-middlewares"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.post_middlewares.push(new_post_middleware?);
                crate::Result::Ok(inner)
            });
        }

        builder
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Builder<B, E> {
    pub fn middleware(self, m: Middleware<B, E>) -> Self {
        self.and_then(move |mut inner| {
            match m {
                Middleware::Pre(middleware) => {
                    inner.pre_middlewares.push(middleware);
                }
                Middleware::Post(middleware) => {
                    inner.post_middlewares.push(middleware);
                }
            }
            crate::Result::Ok(inner)
        })
    }

    pub fn err_handler<H, R>(self, mut handler: H) -> Self
    where
        H: FnMut(crate::Error) -> R + Send + Sync + 'static,
        R: Future<Output = Response<B>> + Send + 'static,
    {
        let handler: ErrHandler<B> = Box::new(move |err: crate::Error| Box::new(handler(err)));

        self.and_then(move |mut inner| {
            inner.err_handler = Some(handler);
            crate::Result::Ok(inner)
        })
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Default
    for Builder<B, E>
{
    fn default() -> Builder<B, E> {
        Builder {
            inner: Ok(BuilderInner {
                post_middlewares: Vec::new(),
                pre_middlewares: Vec::new(),
                routes: Vec::new(),
                err_handler: None,
            }),
        }
    }
}
