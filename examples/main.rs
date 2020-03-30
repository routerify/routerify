extern crate routerify;
use bytes::buf::{Buf, BufExt, BufMut};
use bytes::{Bytes, BytesMut};
use hyper::body::HttpBody;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use routerify::body::StreamBody;
use routerify::prelude::*;
use routerify::utility::middlewares;
use routerify::utility::JsonResponse;
use routerify::{Middleware, Router};
use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio::sync::mpsc;

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
    // JsonResponse::with_error_code(hyper::StatusCode::BAD_REQUEST).into_response()
    JsonResponse::with_success(hyper::StatusCode::CREATED, vec!["USA", "India", "Japan"]).into_response()

    // let file = File::open("./Cargo.toml").await.unwrap();
    // let mut b = StreamBody::new(file);
    //
    // Ok(Response::builder().body(b).unwrap())

    // Ok(Response::new(Body::from("Hello Home")))
}

async fn middleware_logger(req: Request<Body>) -> routerify::Result<Request<Body>> {
    println!("New: {} {} {}", req.remote_addr().unwrap(), req.method(), req.uri());
    Ok(req)
}

async fn handle_home(_: Request<Body>) -> routerify::Result<Response<Body>> {
    Ok(Response::new(Body::from("Hello Home")))
}

async fn type_dirty_test() {
    // let file = File::open("./Cargo.toml").await.unwrap();
    // let mut b = StreamBody::new(file);
    //
    // while let Some(chunk) = b.data().await {
    //     match chunk {
    //         Ok(chunk) => {
    //             let mut r = chunk.reader();
    //             let mut s = String::new();
    //             r.read_to_string(&mut s);
    //             println!("{}", s);
    //         }
    //         Err(err) => println!("{}", err),
    //     }
    // }

    // let (mut tx, mut rx) = mpsc::channel(0);
    //
    // tokio::spawn(async move {
    //     for i in 0..10 {
    //         if let Err(e) = tx.send(10).await {
    //             println!("send error: #{:?}", e);
    //             return;
    //         }
    //     }
    // });
    //
    // while let Some(i) = rx.recv().await {
    //     println!("got = {}", i);
    // }

    const BUF_LEN: usize = 1024 * 1024 * 16;

    let mut buffer = BytesMut::with_capacity(BUF_LEN);
    unsafe {
        buffer.set_len(BUF_LEN);
    }
    println!("BytesMut: {}", buffer.len());

    loop {
        let read: usize = BUF_LEN;
        let bytes = buffer.split_to(read).freeze();
        println!("Bytes: {}", bytes.len());
        drop(bytes);

        println!("BytesMut: {}", buffer.len());
        buffer.reserve(BUF_LEN - buffer.len());
        unsafe {
            buffer.set_len(BUF_LEN);
        }
        println!("BytesMut: {}", buffer.len());

        // std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

#[tokio::main]
async fn main() {
    type_dirty_test().await;

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
