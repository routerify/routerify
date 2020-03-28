extern crate routerify;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use routerify::prelude::*;
use routerify::utility::middlewares;
use routerify::{Middleware, Router};
use std::convert::Infallible;
use std::net::SocketAddr;

lazy_static! {
    static ref ROUTER: Router = Router::builder()
        .middleware(middlewares::query_parser())
        .middleware(Middleware::pre(middleware_logger))
        .middleware(routerify::utility::middlewares::cors_enable_all())
        .get_or_head("/", handle_home)
        .get("/api", handle_api)
        .build()
        .unwrap();
}

async fn handle_api(_req: Request<Body>) -> routerify::Result<Response<Body>> {
    Ok(Response::new(Body::from("Hello Home")))
}

async fn middleware_logger(req: Request<Body>) -> routerify::Result<Request<Body>> {
    println!("New: {} {} {}", req.remote_addr().unwrap(), req.method(), req.uri());
    Ok(req)
}

async fn handle_home(_: Request<Body>) -> routerify::Result<Response<Body>> {
    Ok(Response::new(Body::from("Hello Home")))
}

fn type_dirty_test() {}

#[tokio::main]
async fn main() {
    type_dirty_test();

    let req_service = make_service_fn(|conn: &hyper::server::conn::AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
                Ok::<Response<Body>, Infallible>(routerify::handle_request(&*ROUTER, req, Some(remote_addr)).await)
            }))
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(req_service);

    println!("App is serving on: {}", server.local_addr());
    if let Err(e) = server.await {
        eprintln!("Server Error: {}", e);
    }
}
