use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct State(pub i32);

pub async fn pre_middleware(req: Request<Body>) -> Result<Request<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Pre Data: {}", data);

    Ok(req)
}

pub async fn post_middleware(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, routerify::Error> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Post Data: {}", data);

    Ok(res)
}

pub async fn home_handler(req: Request<Body>) -> Result<Response<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);

    // Ok(Response::new(Body::from(format!("New counter: {}\n", data))))
    Err(routerify::Error::new("Error"))
}

async fn error_handler(err: routerify::Error, req_info: RequestInfo) -> Response<Body> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Error Data: {}", data);

    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router: Router<Body, routerify::Error> = Router::builder()
        .data(State(100))
        .middleware(Middleware::pre(pre_middleware))
        .middleware(Middleware::post_with_info(post_middleware))
        .get("/", home_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
