use crate::data_map::SharedDataMap;
use hyper::{HeaderMap, Method, Request, Uri, Version};
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
}

#[derive(Debug)]
pub(crate) struct RequestInfoInner {
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    version: Version,
}

impl RequestInfo {
    pub(crate) fn new_from_req<B>(req: &Request<B>) -> Self {
        Self {
            req_info_inner: Arc::new(RequestInfoInner {
                headers: req.headers().clone(),
                method: req.method().clone(),
                uri: req.uri().clone(),
                version: req.version(),
            }),
            shared_data_maps: None,
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
}

impl Debug for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.req_info_inner)
    }
}
