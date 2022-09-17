mod session_impls;


/// Implements Unit of Work for accessing the RavenDB server.
#[derive(Debug)]
pub struct DocumentSession {}

impl DocumentSession {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_ravendb_version(&self) -> anyhow::Result<RavenDbVersion> {
        todo!()
    }
}

#[derive(Debug)]
pub struct RavenDbVersion(String);