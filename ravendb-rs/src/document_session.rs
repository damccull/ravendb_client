use tracing::instrument;

mod session_impls;

/// Implements Unit of Work for accessing the RavenDB server.
#[derive(Debug)]
pub struct DocumentSession {}

impl DocumentSession {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[instrument(name = "GET_RAVENDB_VERSION", skip(self))]
    pub async fn get_ravendb_version(&self) -> anyhow::Result<RavenDbVersion> {
        Err(anyhow::anyhow!(
            "Error getting the RavenDB version. There is no code to connect, yet!"
        ))
        //todo!()
    }
}

#[derive(Debug)]
pub struct RavenDbVersion(String);
