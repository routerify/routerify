use core::task::{Context, Poll};
use std::net::SocketAddr;
use std::{convert::Infallible, env, fs, io, pin::Pin, sync, vec::Vec};

use async_stream::stream;
use futures_util::{future::TryFutureExt, stream::Stream};
use hyper::service::{make_service_fn, service_fn, MakeServiceRef};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use rustls_pemfile as pemfile;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};

// A handler for "/" page.
async fn home_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("About page")))
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

#[cfg(feature = "tls-rustls")]
#[tokio::main]
async fn main()  {
    run_server().await;
}

async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/about", about_handler)
        .build()
        .unwrap();
    let service = RouterService::new(router).unwrap();
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    // let service = make_service_fn(|socket: &AddrStream| {
    //     let remote_addr = socket.remote_addr();
    //     async move {
    //         Ok::<_, Infallible>()
    //     }
    // });

    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs("examples/sample.pem")?;
        // Load private key.
        let key = load_private_key("examples/sample.rsa")?;
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| error(format!("{}", e)))?;
        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let tcp = tokio::net::TcpListener::bind(&listen_addr).await?;
    let tls_acceptor = TlsAcceptor::from(tls_cfg.into());
    // Prepare a long-running future stream to accept and serve clients.
    let incoming_tls_stream = stream! {
        loop {
            let (socket, _) = tcp.accept().await?;
            let stream = tls_acceptor.accept(socket).map_err(|e| {
                println!("[!] Voluntary server halt due to client-connection error...");
                // Errors could be handled here, instead of server aborting.
                // Ok(None)
                error(format!("TLS Error: {:?}", e))
            });
            yield stream.await;
        }
    };
    let server = Server::builder(HyperAcceptor {
        acceptor: Box::pin(incoming_tls_stream),
    })
        .serve(service);

    println!("App is running on: {}", listen_addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}

struct HyperAcceptor<'a> {
    acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + 'a>>,
}

impl hyper::server::accept::Accept for HyperAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>, io::Error> {
    // Open certificate file.
    let certfile = fs::File::open(filename).map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificates.
    Ok(pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate".into()))?
        .into_iter()
        .map(rustls::Certificate)
        .collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename).map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::rsa_private_keys(&mut reader).map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(rustls::PrivateKey(keys[0].clone()))
}
