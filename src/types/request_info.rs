use hyper::{Body, HeaderMap, Method, Request, Uri, Version};
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// Represents some information for the incoming request.
///
/// It's used to access request information e.g. headers, method, uri etc for the [Post Middleware](./index.html#post-middleware-with-request-info) and
/// for the [error handling](./index.html#error-handling-with-request-info);
#[derive(Clone)]
pub struct RequestInfo {
    inner: Arc<RequestInfoInner>,
}

#[derive(Clone, Debug)]
struct RequestInfoInner {
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestInfo {
    pub(crate) fn new_from_req(req: &Request<Body>) -> Self {
        let inner = RequestInfoInner {
            headers: req.headers().clone(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            version: req.version(),
        };

        RequestInfo { inner: Arc::new(inner) }
    }

    /// Returns the request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.inner.headers
    }

    /// Returns the request method type.
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    /// Returns the request uri.
    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    /// Returns the request's HTTP version.
    pub fn version(&self) -> Version {
        self.inner.version
    }
}

impl Debug for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
