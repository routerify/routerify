use super::RequestContext;
use crate::data_map::SharedDataMap;
use hyper::{Body, HeaderMap, Method, Request, Uri, Version};
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// Represents some information for the incoming request.
///
/// It's used to access request information e.g. headers, method, uri etc for the [Post Middleware](./index.html#post-middleware-with-request-info) and
/// for the [error handling](./index.html#error-handling-with-request-info);
#[derive(Clone)]
pub struct RequestInfo {
    pub(crate) req_info_inner: Arc<RequestInfoInner>,
    pub(crate) shared_data_maps: Option<Vec<SharedDataMap>>,
    pub(crate) context: RequestContext,
}

#[derive(Debug)]
pub(crate) struct RequestInfoInner {
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestInfo {
    pub(crate) fn new_from_req(req: &Request<Body>, ctx: RequestContext) -> Self {
        let inner = RequestInfoInner {
            headers: req.headers().clone(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: req.version(),
        };

        RequestInfo {
            req_info_inner: Arc::new(inner),
            shared_data_maps: None,
            context: ctx,
        }
    }

    /// Returns the request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.req_info_inner.headers
    }

    /// Returns the request method type.
    pub fn method(&self) -> &Method {
        &self.req_info_inner.method
    }

    /// Returns the request uri.
    pub fn uri(&self) -> &Uri {
        &self.req_info_inner.uri
    }

    /// Returns the request's HTTP version.
    pub fn version(&self) -> Version {
        self.req_info_inner.version
    }

    /// Access data which was shared by the [`RouterBuilder`](./struct.RouterBuilder.html) method
    /// [`data`](./struct.RouterBuilder.html#method.data).
    ///
    /// Please refer to the [Data and State Sharing](./index.html#data-and-state-sharing) for more info.
    pub fn data<T: Send + Sync + 'static>(&self) -> Option<&T> {
        if let Some(ref shared_data_maps) = self.shared_data_maps {
            for shared_data_map in shared_data_maps.iter() {
                if let Some(data) = shared_data_map.inner.get::<T>() {
                    return Some(data);
                }
            }
        }

        None
    }

    /// Access data from the request context.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams, Middleware, RequestInfo};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Request, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .middleware(Middleware::pre(|req: Request<Body>| async move {
    ///         req.set_context("example".to_string());
    ///
    ///         Ok(req)
    ///     }))
    ///     .middleware(Middleware::post_with_info(|res, req_info: RequestInfo| async move {
    ///         let text = req_info.context::<String>().unwrap();
    ///         println!("text is {}", text);
    ///
    ///         Ok(res)
    ///     }))
    ///     .get("/hello", |req| async move {
    ///         let text = req.context::<String>().unwrap();
    ///
    ///         Ok(Response::new(Body::from(format!("Hello from : {}", text))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn context<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
        self.context.get::<T>()
    }
}

impl Debug for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.req_info_inner)
    }
}
