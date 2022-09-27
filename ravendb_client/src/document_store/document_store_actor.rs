use std::{collections::HashMap, net::SocketAddr};

use anyhow::Context;
use rand::seq::IteratorRandom;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Identity, Url,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{instrument, Span};
use uuid::Uuid;

use crate::{
    cluster_topology::ClusterTopologyInfo,
    raven_command::{RavenCommand, RavenCommandVariant},
    CertificatePlaceholder, Conventions, DnsOverrides, DocumentStoreError,
    DocumentStoreInitialConfiguration, DocumentStoreMessage,
};

pub struct DocumentStoreActor {
    //_async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    client_identity: Option<reqwest::Identity>,
    dns_overrides: Option<DnsOverrides>,
    _conventions: Option<Conventions>,
    _database_name: Option<String>,
    proxy_address: Option<String>,
    receiver: mpsc::Receiver<DocumentStoreMessage>,
    _trust_store: Option<CertificatePlaceholder>,
    topology_info: ClusterTopologyInfo,
    topology_updater: Option<JoinHandle<()>>,
}
impl DocumentStoreActor {
    pub fn new(
        receiver: mpsc::Receiver<DocumentStoreMessage>,
        initial_config: DocumentStoreInitialConfiguration,
    ) -> Self {
        Self {
            //_async_document_id_generator: initial_config.async_document_id_generator,
            _conventions: Default::default(),
            client_identity: initial_config.client_identity,
            _database_name: initial_config.database_name,
            dns_overrides: initial_config.dns_overrides,
            proxy_address: initial_config.proxy_address,
            receiver,
            _trust_store: Some(CertificatePlaceholder),
            topology_info: initial_config.cluster_topology,
            topology_updater: None,
        }
    }

    /// Message handler for the DocumentStoreActor
    #[instrument(
        level = "debug",
        name = "DocumentStore Actor - Handle Message",
        skip(self),
        fields(correlation_id)
    )]
    async fn handle_message(&mut self, msg: DocumentStoreMessage) {
        // Apply a correlation id to all child spans of this message handler
        Span::current().record("correlation_id", Uuid::new_v4().to_string());
        match msg {
            DocumentStoreMessage::ExecuteRavenCommand {
                raven_command,
                respond_to,
            } => {
                // Define a struct to hold data for the tokio tasks
                struct RavenCommandTaskData {
                    dns_overrides: Option<DnsOverrides>,
                    identity: Option<Identity>,
                    node_url: Url,
                    proxy_address: Option<String>,
                    topology_respond_to:
                        oneshot::Sender<Result<Option<ClusterTopologyInfo>, DocumentStoreError>>,
                    topology_etag: i64,
                    topology_updater_running: bool,
                }

                // Create a channel to handle topology updates if we need it
                let (topo_tx, topo_rx) = oneshot::channel();
                // Can't call this inside the task, but since the topology task might need it, cache a url here
                let node_url = match self.get_server_address() {
                    Ok(url) => url,
                    Err(e) => {
                        tracing::error!("Unable to get url for topology update. Caused by: {}", e);
                        return;
                    }
                };

                // Determine if a topology update is already running.
                // If this is None, then the updater isn't running. If Some, ask if it's finished.
                let topology_updater_running = match &self.topology_updater {
                    Some(handle) => !handle.is_finished(),
                    None => false,
                };

                // Define the task data
                let taskdata = RavenCommandTaskData {
                    dns_overrides: self.dns_overrides.clone(),
                    identity: self.client_identity.clone(),
                    node_url,
                    proxy_address: self.proxy_address.clone(),
                    topology_respond_to: topo_tx,
                    topology_etag: self.topology_info.etag,
                    topology_updater_running,
                };

                // Spawn a task to do the request
                tokio::spawn(async move {
                    let result = DocumentStoreActor::send_raven_command_request_to_server(
                        taskdata.identity.clone(),
                        taskdata.dns_overrides.clone(),
                        taskdata.proxy_address.clone(),
                        raven_command,
                        taskdata.topology_etag,
                    )
                    .await;

                    // Spin off a task to update the topology if needed
                    match &result {
                        Ok(response) => {
                            // Only spawn the update task if one isn't already running
                            if !taskdata.topology_updater_running {
                                let headers = response.headers().clone();
                                tokio::spawn(async move {
                                    DocumentStoreActor::refresh_topology_task(
                                        taskdata.identity,
                                        taskdata.dns_overrides,
                                        headers,
                                        taskdata.proxy_address,
                                        taskdata.topology_respond_to,
                                        taskdata.topology_etag,
                                        taskdata.node_url,
                                    )
                                    .await
                                });
                            }
                        }
                        Err(_) => {}
                    };

                    // Send the result back to the caller
                    let _ = respond_to.send(result);
                });

                // Await the topology update here
                match topo_rx.await {
                    Ok(msg) => match msg {
                        Ok(topology) => {
                            if let Some(t) = topology {
                                self.topology_info = t;
                                tracing::info!("Topology updated successfully.");
                            } else {
                                tracing::debug!("No topology update needed.")
                            }
                        }
                        Err(err) => {
                            tracing::error!("Unable to update topology. Caused by: {}", err);
                        }
                    },
                    Err(e) => {
                        tracing::error!("Unable to update topology. Caused by: {}", e);
                    }
                }
            }
            DocumentStoreMessage::GetServerAddress { respond_to } => {
                let result = self.get_server_address();
                let _ = respond_to.send(result);
            }
        }
    }

    /// Checks to see if the server is indicating a need for refreshing the topology and refreshes it if so.
    #[instrument(level = "debug", skip(client_identity, respond_to))]
    async fn refresh_topology_task(
        client_identity: Option<Identity>,
        dns_overrides: Option<DnsOverrides>,
        headers: HeaderMap,
        proxy_address: Option<String>,
        respond_to: oneshot::Sender<Result<Option<ClusterTopologyInfo>, DocumentStoreError>>,
        topology_etag: i64,
        url: Url,
    ) {
        tracing::trace!("Attempting topology update");
        tracing::trace!("Request headers are: {:#?}", &headers);
        // Check if the Refresh-Topology response header exists and is false, or doesn't exist
        // and return early
        if let Some(refresh) = headers.get("Refresh-Topology".to_lowercase()) {
            if refresh.to_str().unwrap_or("false") == "true" {
                tracing::trace!("Found key `{:?}`", refresh);
                let get_topology = RavenCommand {
                    base_server_url: url,
                    command: RavenCommandVariant::GetClusterTopology,
                };
                let result = match DocumentStoreActor::send_raven_command_request_to_server(
                    client_identity,
                    dns_overrides,
                    proxy_address,
                    get_topology,
                    topology_etag,
                )
                .await
                {
                    Ok(response) => response,
                    Err(e) => {
                        // Return early
                        let _ = respond_to.send(Err(DocumentStoreError::UnexpectedError(e)));
                        return;
                    }
                };

                let result = match result.json::<ClusterTopologyInfo>().await {
                    Ok(topo) => topo,
                    Err(e) => {
                        // Return early
                        let _ = respond_to.send(Err(DocumentStoreError::UnexpectedError(
                            anyhow::anyhow!(
                                "Unable to deserialize cluster topology information. Caused by: {}",
                                e
                            ),
                        )));
                        return;
                    }
                };

                let _ = respond_to.send(Ok(Some(result)));
                return;
            }
        }
        // No update needed
        tracing::trace!("Refresh-Topology header not present or 'false'. Not performing update.");
        let _ = respond_to.send(Ok(None));
    }

    #[instrument(level = "debug", skip(client_identity))]
    async fn send_raven_command_request_to_server(
        client_identity: Option<Identity>,
        dns_overrides: Option<DnsOverrides>,
        proxy_address: Option<String>,
        raven_command: RavenCommand,
        topology_etag: i64,
    ) -> anyhow::Result<reqwest::Response> {
        let mut client = reqwest::Client::builder();

        if let Some(identity) = client_identity.clone() {
            client = client.identity(identity).use_rustls_tls();
        }

        // Convert Option<HashMap<String, IpAddr>> into HashMap<String,SocketAddr>
        let overrides = dns_overrides
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, SocketAddr::new(v, 0)))
            .collect::<HashMap<String, SocketAddr>>();

        for (domain, address) in overrides {
            tracing::trace!(
                "Adding `{}->{}` to dns overrides for this request.",
                domain,
                address
            );
            client = client.resolve(domain.as_str(), address);
        }

        if let Some(proxy) = proxy_address {
            tracing::trace!("Proxy set to `{}`", &proxy);
            client = client.proxy(reqwest::Proxy::http(proxy)?);
        } else {
            tracing::trace!("No proxy define. Using system settings.");
        }

        let client = client.build()?;

        let mut request = raven_command.get_http_request()?;
        let headerval = HeaderValue::from_str(topology_etag.to_string().as_str())?;
        request.headers_mut().append("Topology-Etag", headerval);
        tracing::trace!("Request Headers: {:#?}", &request.headers());
        let response = client.execute(request).await?;

        Ok(response)
    }

    #[instrument(
        level = "debug",
        name = "DocumentStore Actor - Get Server Address",
        skip(self)
    )]
    fn get_server_address(&self) -> anyhow::Result<Url> {
        let url = self
            .topology_info
            .topology
            .all_nodes
            .values()
            .choose(&mut rand::thread_rng())
            .context("Urls list is empty")
            .cloned();
        if let Ok(u) = &url {
            tracing::debug!("Selected Url: {}", u);
        }
        url
    }
}

#[instrument(level = "debug", name = "Running Document Store Actor", skip(actor))]
pub async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}
