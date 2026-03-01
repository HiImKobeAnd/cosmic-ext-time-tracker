use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not authorized")]
    NotAuthorized,
    #[error("failed to authenticate")]
    FailedToAuthenticate,
    #[error("invalid URL `{0}`")]
    InvalidURL(#[from] url::ParseError),
    #[error("network error")]
    NetworkError(#[from] reqwest::Error),
    #[error("wrong time entry context given")]
    WrongTimeEntryContext,
    #[error("wrong project context given")]
    WrongProjectContext,
}
