use hyper::Method;

pub(crate) const HEADER_NAME_X_POWERED_BY: &'static str = "x-powered-by";
pub(crate) const HEADER_VALUE_X_POWERED_BY: &'static str = concat!("Routerify v", env!("CARGO_PKG_VERSION"));

pub(crate) const ALL_POSSIBLE_HTTP_METHODS: [Method; 9] = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
    Method::CONNECT,
    Method::HEAD,
    Method::OPTIONS,
    Method::TRACE,
];
