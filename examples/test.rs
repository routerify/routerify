use hyper::{Body, Response, Server};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use std::net::SocketAddr;

fn router() -> Router<Body, routerify::Error> {
    let mut builder = Router::builder();

    for i in 0..3000_usize {
        builder = builder.middleware(
            Middleware::pre_with_path(format!("/abc-{}", i), move |req| async move {
                // println!("PreMiddleware: {}", format!("/abc-{}", i));
                Ok(req)
            })
            .unwrap(),
        );

        builder = builder.get(format!("/abc-{}", i), move |req| async move {
            // println!("Route: {}, params: {:?}", format!("/abc-{}", i), req.params());
            Ok(Response::new(Body::from(format!("/abc-{}", i))))
        });

        builder = builder.middleware(
            Middleware::post_with_path(format!("/abc-{}", i), move |res| async move {
                // println!("PostMiddleware: {}", format!("/abc-{}", i));
                Ok(res)
            })
            .unwrap(),
        );
    }

    builder.build().unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 89.154971ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 89.338824ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 90.20536ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 87.520673ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 90.237893ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 87.364556ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 87.477891ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 86.946817ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 86.86861ms
// PreMiddleware: /abc-9999
// Route: /abc-9999
// PostMiddleware: /abc-9999
// Process time: 88.663156ms
