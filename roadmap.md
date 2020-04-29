# Roadmap for v2.0.0

## Features can be added
* Improve router logic by not truncating the path on every `.route` level.
* Consumer should use Router as an instance instead of static reference; Create a hyper `Service` to solve it.
* Implement own `Request` and `Response` to allow efficient data streaming with `StreamBody` as the following:

```rust
struct HttpRequest(Request<Body>);

impl Deref for HttpRequest {
    type Target = Request<Body>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct HttpResponse(Response<StreamBody>);

impl HttpResponse {
    fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder(Builder::new())
    }
}

impl Deref for HttpResponse {
    type Target = Response<StreamBody>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct HttpResponseBuilder(Builder);

impl HttpResponseBuilder {
    fn body<B: Into<StreamBody>>(self, body: B) -> http::Result<HttpResponse> {
        self.0.body(body.into()).map(|raw| HttpResponse(raw))
    }
}

impl Deref for HttpResponseBuilder {
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpResponseBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

async fn handler(req: HttpRequest) -> Result<HttpResponse, Infallible> {
    println!("{}", req.uri().path());

    let (mut w, body) = StreamBody::channel();
    let content = "Hello World";

    tokio::spawn(async move {
        w.write_all(content.as_bytes()).await.unwrap();
    });

    Ok(HttpResponse::builder()
        // It will throw error as header returns http::response::Builder not HttpResponseBuilder.
        // You have to create the wrapper methods to HttpResponseBuilder instead of implementing Deref and DerefMut.
        .header("Content-Length", content.len().to_string())
        .body(body)
        .unwrap())
}

async fn hyper_handler(req: Request<Body>) -> Result<Response<StreamBody>, Infallible> {
    handler(HttpRequest(req)).await.map(|resp| resp.0)
}
```

Note: You can add wrapper functions to `HttpRequest`, `HttpResponse` and `HttpResponseBuilder` instead of implementing `Deref` and `DerefMut` to restrict some functionalities e.g. not allowing mutation to the wrapper `Request<Body>` or allowing chaining to the `HttpResponseBuilder`.