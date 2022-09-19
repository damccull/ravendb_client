use tracing::instrument;
use url::Url;

use crate::{
    raven_command::{HttpMethod, RavenCommand, RavenCommandOption},
    DocumentStore,
};

mod session_impls;

/// Implements Unit of Work for accessing the RavenDB server.
#[derive(Debug)]
pub struct DocumentSession {
    document_store: DocumentStore,
}

impl DocumentSession {
    #[allow(clippy::new_without_default)]
    pub fn new(document_store: DocumentStore) -> Self {
        Self { document_store }
    }

    #[instrument(name = "GET_RAVENDB_CLUSTER_TOPOLOGY", skip(self))]
    pub async fn get_cluster_topology(&self) -> anyhow::Result<String> {
        //let server = self.document_store.get_urls();
        let raven_command = RavenCommand {
            base_server_url: Url::parse("https://a.free.damccull.ravendb.cloud")?,
            //endpoint: "/cluster/topology".to_string(),
            http_method: HttpMethod::Get,
            headers: Vec::new(),
            command: RavenCommandOption::GetClusterTopology,
        };

        let response = self
            .document_store
            .execute_raven_command(raven_command)
            .await?;

        let body = response.text().await?;
        Ok(body)
        // Err(anyhow::anyhow!(
        //     "Error getting the RavenDB version. There is no code to connect, yet!"
        // ))
        //
    }
}

#[derive(Debug)]
pub struct RavenDbVersion(String);
