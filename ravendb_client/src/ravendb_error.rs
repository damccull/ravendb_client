use crate::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum RavenDbError {
    #[error("Invalid authorization, ensure valid certificate supplied")]
    BadAuthorization,
    #[error("Database `{0}` does not exist")]
    DatabaseDoesNotExist(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl std::fmt::Debug for RavenDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
