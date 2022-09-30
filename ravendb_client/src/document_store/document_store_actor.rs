use std::{collections::HashMap, net::SocketAddr};

use anyhow::Context;
use rand::seq::IteratorRandom;
use reqwest::{header::HeaderValue, Identity, Url};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{instrument, Span};
use uuid::Uuid;

use crate::{
    cluster_topology::ClusterTopologyInfo,
    document_conventions::DocumentConventions,
    raven_command::{RavenCommand, RavenCommandVariant},
    request_executor::RequestExecutor,
    CertificatePlaceholder, DnsOverrides, DocumentStoreError, DocumentStoreInitialConfiguration,
    DocumentStoreMessage,
};

pub struct DocumentStoreActor {
    client_identity: Option<reqwest::Identity>,
    dns_overrides: Option<DnsOverrides>,
    conventions: DocumentConventions,
    database_name: Option<String>,
    proxy_address: Option<String>,
    receiver: mpsc::Receiver<DocumentStoreMessage>,
    /// Allows the actor to receive messages from itself.
    receiver_internal: mpsc::Receiver<DocumentStoreMessage>,
    request_executors: HashMap<String, RequestExecutor>,
    /// Allows the actor to send messages to itself.
    sender_internal: mpsc::Sender<DocumentStoreMessage>,
    _trust_store: Option<CertificatePlaceholder>,
    topology_info: ClusterTopologyInfo,
    topology_updater: Option<JoinHandle<Result<ClusterTopologyInfo, DocumentStoreError>>>,
}
impl DocumentStoreActor {
    pub fn new(
        receiver: mpsc::Receiver<DocumentStoreMessage>,
        initial_config: DocumentStoreInitialConfiguration,
    ) -> Self {
        let (tx, rx) = mpsc::channel(10);
        Self {
            conventions: DocumentConventions::default(),
            client_identity: initial_config.client_identity,
            database_name: initial_config.database_name,
            dns_overrides: initial_config.dns_overrides,
            proxy_address: initial_config.proxy_address,
            receiver,
            receiver_internal: rx,
            request_executors: HashMap::default(),
            sender_internal: tx,
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
                let dns_overrides = self.dns_overrides.clone();
                let identity = self.client_identity.clone();
                let proxy_address = self.proxy_address.clone();
                let topology_etag = self.topology_info.etag;
                let sender_internal = self.sender_internal.clone();

                // Spawn a task to do the request
                tokio::spawn(async move {
                    let result = DocumentStoreActor::send_raven_command_request_to_server(
                        identity.clone(),
                        dns_overrides.clone(),
                        proxy_address.clone(),
                        raven_command,
                        topology_etag,
                    )
                    .await;

                    if let Ok(response) = &result {
                        if let Some(value) =
                            response.headers().get("Refresh-Topology".to_lowercase())
                        {
                            if value.to_str().unwrap_or("false") == "true" {
                                if let Err(e) = sender_internal
                                    .send(DocumentStoreMessage::UpdateTopology)
                                    .await
                                {
                                    tracing::error!(
                                        "Could not send internal message to request topology update. Caused by: {}",
                                         e
                                    );
                                }
                            }
                        }
                    }

                    // Send the result back to the caller
                    let _ = respond_to.send(result);
                });
            }
            DocumentStoreMessage::GetDatabase { respond_to } => {
                let _ = respond_to.send(self.database_name.clone());
            }
            DocumentStoreMessage::GetRequestExecutor {
                database_name,
                respond_to,
            } => {
                let result = self.get_request_executor(database_name).await;
                let _ = respond_to.send(result);
            }
            DocumentStoreMessage::GetServerAddress { respond_to } => {
                let result = self.get_server_address().await;
                let _ = respond_to.send(result);
            }
            DocumentStoreMessage::UpdateTopology => {
                tracing::debug!("Updating topology.");
                match self.refresh_topology().await {
                    Ok(_) => tracing::debug!("Topology update downloaded, awaiting store."),
                    Err(e) => {
                        tracing::error!(
                            "There was an error updating the topology. Caused by: {}",
                            e
                        );
                    }
                }
            }
        }
    }

    /// Refreshes the cluster topology.
    #[instrument(level = "debug", skip(self))]
    async fn refresh_topology(&mut self) -> Result<(), DocumentStoreError> {
        // Determine if a topology update is already running and cancel if it is.
        if self
            .topology_updater
            .as_ref()
            .map(|x| !x.is_finished())
            .unwrap_or(false)
        {
            tracing::debug!(
                "Topology update already running. Canceling to avoid duplication of effort."
            );
            return Ok(());
        }

        tracing::trace!("Attempting topology update");
        // Check if the Refresh-Topology response header exists and is false, or doesn't exist
        // and return early
        let get_topology = RavenCommand {
            base_server_url: self.get_server_address().await?,
            command: RavenCommandVariant::GetClusterTopology,
        };

        let client_identity = self.client_identity.clone();
        let dns_overrides = self.dns_overrides.clone();
        let proxy_address = self.proxy_address.clone();
        let etag = self.topology_info.etag;

        // Kick off an async task to actually do the update. Store the joinhandle for later use.
        // The results of this will be dealt with in the get_server_address function.
        self.topology_updater = Some(tokio::spawn(async move {
            let result = match DocumentStoreActor::send_raven_command_request_to_server(
                client_identity,
                dns_overrides,
                proxy_address,
                get_topology,
                etag,
            )
            .await
            {
                Ok(response) => response,
                Err(e) => {
                    // Return early
                    return Err(DocumentStoreError::UnexpectedError(anyhow::anyhow!(
                        "Unable to send command to server. Caused by: {}",
                        e
                    )));
                }
            };

            let result = match result.json::<ClusterTopologyInfo>().await {
                Ok(topo) => topo,
                Err(e) => {
                    // Return early
                    return Err(DocumentStoreError::UnexpectedError(anyhow::anyhow!(
                        "Unable to deserialize cluster topology information. Caused by: {}",
                        e
                    )));
                }
            };
            Ok(result)
        }));

        Ok(())
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
            tracing::trace!("No proxy defined. Using system settings.");
        }

        let client = client.build()?;

        let mut request = raven_command.get_http_request()?;
        let headerval = HeaderValue::from_str(topology_etag.to_string().as_str())?;
        request.headers_mut().append("Topology-Etag", headerval);
        tracing::trace!("Request Headers: {:#?}", &request.headers());
        let response = client.execute(request).await?;

        Ok(response)
    }

    /// See doc comments for [`DocumentStore`](crate::DocumentStore::get_request_executor)
    #[instrument(level = "debug", skip(self))]
    async fn get_request_executor(
        &mut self,
        database: Option<String>,
    ) -> std::result::Result<RequestExecutor, DocumentStoreError> {
        // Get the database name that was passed in, or from the document store
        let database = match database {
            Some(db) => db,
            None => match self.database_name.as_ref() {
                Some(db) => db.clone(),
                None => {
                    return Err(DocumentStoreError::UnexpectedError(anyhow::anyhow!(
                        "Unable to determine which database to operate on"
                    )));
                }
            },
        };

        // See if there is a stored executor for the database
        if let Some(executor) = self.request_executors.get(&database) {
            return Ok(executor.clone());
        }

        // Creates a RequestExecutor for a normal cluster
        let create_request_executor = || -> RequestExecutor {
            // TODO: Figure out how to allow the request executor to publish events
            RequestExecutor::new()
        };

        // Creates a request executor for a single, specific server, ignoring topology
        let create_request_executor_for_single_node = || -> RequestExecutor {
            // TODO: Figure out how to allow the request executor to publish events
            RequestExecutor::new_for_single_node_with_configuration_updates()
        };

        let executor = if self.conventions.disable_topology_updates() {
            create_request_executor_for_single_node()
        } else {
            create_request_executor()
        };

        // Clone the executor handle store it in the document store
        self.request_executors.insert(database, executor.clone());

        // Send the executor handle back to the requestor
        Ok(executor)
    }

    #[instrument(
        level = "debug",
        name = "DocumentStore Actor - Get Server Address",
        skip(self)
    )]
    async fn get_server_address(&mut self) -> anyhow::Result<Url> {
        // Check if there is a completed JoinHandle for the topology updater. If so, try to extract
        // its data and set it into the actor's topology_info field.
        if let Some(updater) = self.topology_updater.as_ref() {
            if updater.is_finished() {
                let handle = self.topology_updater.take();
                if let Some(handle) = handle {
                    // Double question mark is to unwrap both the Result and its own contents.
                    self.topology_info = handle.await??;
                    tracing::info!("Topology updated and stored.");
                }
            }
        }

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
    let mut topology_update_timer = tokio::time::interval(tokio::time::Duration::from_secs(5));
    loop {
        tokio::select! {
            _ = topology_update_timer.tick() => {
                tracing::debug!("Updating topology via timer.");
                let _= actor.sender_internal.send(DocumentStoreMessage::UpdateTopology).await;
            },
            opt_msg = actor.receiver.recv() => {
                let msg = match opt_msg {
                    Some(msg) => msg,
                    None => break,
                };
                actor.handle_message(msg).await;
            },
            Some(msg) = actor.receiver_internal.recv() => {
                actor.handle_message(msg).await;
            }
        }
    }
}
