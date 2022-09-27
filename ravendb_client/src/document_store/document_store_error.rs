use crate::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum DocumentStoreError {
    #[error("No URLs were supplied and a document store can't exist without at least one")]
    MissingUrlsError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl std::fmt::Debug for DocumentStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
