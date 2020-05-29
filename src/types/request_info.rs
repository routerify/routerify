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
    pub(crate) inner: Arc<RequestInfoInner>,
    pub(crate) shared_data_map: Option<SharedDataMap>,
}

#[derive(Debug)]
pub(crate) struct RequestInfoInner {
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

        RequestInfo {
            inner: Arc::new(inner),
            shared_data_map: None,
        }
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

    /// Access data which was shared by the [`RouterBuilder`](./struct.RouterBuilder.html) method
    /// [`data`](./struct.RouterBuilder.html#method.data).
    ///
    /// Please refer to the [Data and State Sharing](./index.html#data-and-state-sharing) for more info.
    pub fn data<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.shared_data_map
            .as_ref()
            .and_then(|data_map| data_map.inner.get::<T>())
    }
}

impl Debug for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
