/// The error type used by the `Routerify` library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't decode the request path as UTF8")]
    DecodeRequestPath(#[source] std::str::Utf8Error),

    #[error("Couldn't create router RegexSet")]
    CreateRouterRegexSet(#[source] regex::Error),

    #[error("Could not create an exact match regex for the route path: {1}")]
    GenerateExactMatchRegex(#[source] regex::Error, String),

    #[error("Could not create an exact match regex for the route path: {1}")]
    GeneratePrefixMatchRegex(#[source] regex::Error, String),

    #[error("No handlers added to handle non-existent routes. Tips: Please add an '.any' route at the bottom to handle any routes.")]
    HandleNonExistentRoute,

    #[error("A route was unable to handle the pre middleware request")]
    HandlePreMiddlewareRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("A route was unable to handle the request for target: {1}")]
    HandleRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>, String),

    #[error("One of the post middlewares (without info) couldn't process the response")]
    HandlePostMiddlewareWithoutInfoRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("One of the post middlewares (with info) couldn't process the response")]
    HandlePostMiddlewareWithInfoRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("A route was unable to handle the request due to the maximum size being exceeded {0} {1}")]
    HandleOverSizeRequest(u64, u64),
}
