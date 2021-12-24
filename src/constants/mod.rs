use hyper::Method;

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
