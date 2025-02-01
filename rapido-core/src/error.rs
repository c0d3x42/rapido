use thiserror::Error;

#[derive(Debug, Error)]
pub enum RapidoError {
    #[error("db failed")]
    DatabaseInitFailed(String),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error("not done")]
    NotImplemented
}
