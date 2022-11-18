/// Query related errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}
