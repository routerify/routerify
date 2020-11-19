/// The error type used by the `Routerify` library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't decode the request path as UTF8")]
    DecodeRequestPath(#[source] std::str::Utf8Error),

    #[error("Couldn't create router RegexSet")]
    CreateRouterRegexSet(#[source] regex::Error),

    #[error("Could not create an exact match regex for the route path")]
    GenerateExactMatchRegex(#[source] regex::Error),

    #[error("Could not create an exact match regex for the route path")]
    GeneratePrefixMatchRegex(#[source] regex::Error),

    #[error("One of the pre middlewares couldn't process the request")]
    ProcessPreMiddleware(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("No handlers added to handle non-existent routes. Tips: Please add an '.any' route at the bottom to handle any routes.")]
    HandleNonExistentRoute,

    #[error("One of the post middlewares couldn't process the response")]
    ProcessPostMiddleware(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("One of the routes couldn't handle the request")]
    HandleRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}
