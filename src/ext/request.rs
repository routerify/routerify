use crate::types::{RequestMeta, RouteParams};
use hyper::{body::HttpBody, Request};
use std::net::SocketAddr;

pub trait RequestExt {
    fn params(&self) -> &RouteParams;
    fn param<P: Into<String>>(&self, param_name: P) -> Option<&String>;
    fn remote_addr(&self) -> SocketAddr;
}

impl<B: HttpBody + Send + Sync + Unpin + 'static> RequestExt for Request<B> {
    fn params(&self) -> &RouteParams {
        self.extensions()
            .get::<RequestMeta>()
            .and_then(|meta| meta.route_params())
            .expect("Routerify: No RouteParams added while processing request")
    }

    fn param<P: Into<String>>(&self, param_name: P) -> Option<&String> {
        self.params().get(&param_name.into())
    }

    fn remote_addr(&self) -> SocketAddr {
        self.extensions()
            .get::<RequestMeta>()
            .and_then(|meta| meta.remote_addr())
            .copied()
            .expect("Routerify: No remote address added while processing request")
    }
}
