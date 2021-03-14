use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::net::SocketAddr;

pub struct State(pub i32);

pub async fn pre_middleware(req: Request<Body>) -> Result<Request<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Pre Data: {}", data);
    println!("Pre Data2: {:?}", req.data::<u32>());

    Ok(req)
}

pub async fn post_middleware(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, routerify::Error> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Post Data: {}", data);

    Ok(res)
}

pub async fn home_handler(req: Request<Body>) -> Result<Response<Body>, routerify::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Route Data: {}", data);
    println!("Route Data2: {:?}", req.data::<u32>());

    Err(routerify::Error::new("Error"))
}

async fn error_handler(err: routerify::RouteError, req_info: RequestInfo) -> Response<Body> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Error Data: {}", data);
    println!("Error Data2: {:?}", req_info.data::<u32>());

    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router2() -> Router<Body, routerify::Error> {
    Router::builder()
        .data(111_u32)
        .get("/a", |req| async move {
            println!("Router2 Data: {:?}", req.data::<&str>());
            println!("Router2 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router2 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Body::from("Hello world!")))
        })
        .build()
        .unwrap()
}

fn router3() -> Router<Body, routerify::Error> {
    Router::builder()
        .data(555_u32)
        .get("/h/g/j", |req| async move {
            println!("Router3 Data: {:?}", req.data::<&str>());
            println!("Router3 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router3 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Body::from("Hello world!")))
        })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router: Router<Body, routerify::Error> = Router::builder()
        .data(State(100))
        .scope("/r", router2())
        .scope("/bcd", router3())
        .data("abcd")
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
