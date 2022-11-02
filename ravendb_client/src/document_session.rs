use tracing::instrument;

use crate::{
    cluster_topology::ClusterTopologyInfo,
    raven_command::{RavenCommand, RavenCommandVariant},
    DocumentStore,
};

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

    #[instrument(level = "info", name = "Get Cluster Topology", skip(self))]
    pub async fn get_cluster_topology(&self) -> anyhow::Result<ClusterTopologyInfo> {
        let raven_command = RavenCommand {
            base_server_url: self.document_store.get_server_address().await?,
            command: RavenCommandVariant::GetClusterTopology,
        };

        // let response = self
        //     .document_store
        //     .execute_raven_command(raven_command)
        //     .await?;

        // let topology = response.json::<ClusterTopologyInfo>().await?;
        // tracing::info!("Cluster topology downloaded");
        // Ok(topology)
        todo!()
    }

    #[instrument(level = "info", name = "Get All Documents for Database", skip(self))]
    pub async fn get_all_documents_for_database(
        &self,
        database: &str,
        page_size: Option<i64>,
        start: Option<i64>,
    ) -> anyhow::Result<String> {
        let raven_command = RavenCommand {
            base_server_url: self.document_store.get_server_address().await?,
            command: RavenCommandVariant::GetAllDocumentsFromDatabase {
                database: database.to_string(),
                page_size,
                start,
            },
        };

        // let response = self
        //     .document_store
        //     .execute_raven_command(raven_command)
        //     .await?;

        // let topology = response.text().await?;
        // tracing::info!("Got documents from database `{}`", database);
        // Ok(topology)
        todo!()
    }
}

#[derive(Debug)]
pub struct RavenDbVersion(String);
