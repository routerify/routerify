use self::support::{into_text, serve};
use hyper::{Body, Client, Request, Response};
use routerify::Router;

mod support;

#[tokio::test]
async fn test_hello_world() {
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/", |_| async move { Ok(Response::new("Hello world".into())) })
        .build()
        .unwrap();

    let serve = serve(router).await;

    let resp = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let txt = into_text(resp.into_body()).await;

    assert_eq!(txt, "Hello world".to_owned());

    serve.shutdown();
}
