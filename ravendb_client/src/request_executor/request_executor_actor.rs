use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use rand::{seq::IteratorRandom, thread_rng};
use reqwest::{header::HeaderValue, Identity, Url};
use tokio::sync::mpsc;
use tracing::{instrument, Span};
use uuid::Uuid;

use crate::{
    database_topology::{self, DatabaseTopology},
    document_conventions::DocumentConventions,
    node_selector::NodeSelector,
    raven_command::RavenCommand,
    server_node::ServerNode,
    DnsOverrides,
};

use super::{RequestExecutorError, RequestExecutorMessage};

pub struct RequestExecutorActor {
    /// Allows the server to warn if [`DocumentStore`] is being recreated too many times
    /// instead of once per application. RequestExecutor should be cached and reused, so
    /// this shouldn't change after initialization.
    application_id: Uuid,
    conventions: DocumentConventions,
    database: String,
    database_topology: Option<DatabaseTopology>,
    dns_overrides: DnsOverrides,
    identity: Option<Identity>,
    last_known_urls: Vec<Url>,
    node_selector: Option<NodeSelector>,
    proxy_address: Option<String>,
    receiver: mpsc::Receiver<RequestExecutorMessage>,
    receiver_internal: mpsc::Receiver<RequestExecutorMessage>,
    /// Cached http client. Clone this into tokio::spawn() for each request, it's cheap.
    reqwest_client: reqwest::Client,
    sender_internal: mpsc::Sender<RequestExecutorMessage>,
    /// Whether or not to run speed tests
    run_speed_test: bool,
    /// Holds the topology
    topology: Option<DatabaseTopology>,
}

impl RequestExecutorActor {
    pub(crate) fn new(
        receiver: mpsc::Receiver<RequestExecutorMessage>,
        database: String,
        identity: Option<Identity>,
        initial_urls: Vec<Url>,
        dns_overrides: DnsOverrides,
        proxy_address: Option<String>,
        conventions: DocumentConventions,
    ) -> Self {
        // Reqwest client maintains an internal connection pool. Reuse it so long as this
        // RequestExecutor lives.
        let reqwest_client = reqwest::Client::new();

        // Create internal messaging channel
        let (sender_internal, receiver_internal) = mpsc::channel(10);

        // Get the initial topology
        //let database_topology = self.initial_topology_update();

        // if let Some(identity) = client_identity.clone() {
        //                 client = client.identity(identity).use_rustls_tls();
        //             }

        //TODO: Kick off first topology update

        Self {
            application_id: Uuid::new_v4(),
            conventions,
            database,
            database_topology: None,
            dns_overrides,
            identity: Option::default(),
            last_known_urls: Vec::default(),
            node_selector: Option::default(),
            proxy_address,
            receiver,
            receiver_internal,
            reqwest_client,
            sender_internal,
            run_speed_test: false,
            topology: None,
        }
    }
    async fn handle_message(&mut self, msg: RequestExecutorMessage) {
        // Apply a correlation id to all child spans of this message handler
        Span::current().record("correlation_id", Uuid::new_v4().to_string());
        match msg {
            RequestExecutorMessage::ExecuteRequest {
                respond_to,
                request,
            } => {
                //TODO: Nuke this and wait for topology to be done, maybe.
                let Some(topology) = self.database_topology else {
                    // Database doesn't exist yet so send the caller a message to tell them
                    let _ = respond_to.send(Err(RequestExecutorError::UnexpectedError(anyhow::anyhow!("Unable to get topology, initial update not yet finished"))));
                    return;
                };

                let dns_overrides = self.dns_overrides.clone();
                let identity = self.identity.clone();
                let proxy_address = self.proxy_address.clone();
                let topology_etag = topology.etag;
                let sender_internal = self.sender_internal.clone();

                // Spawn a task to do the request
                tokio::spawn(async move {
                    let result = send_raven_command_request_to_server(
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
                                    .send(RequestExecutorMessage::UpdateTopology)
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
            RequestExecutorMessage::InitialUpdateTopology { initial_urls } => {
                //TODO: Change this to handle subsequent topology updates, or consider handling both initial and subsequent in same fn
                // let result = self
                //     .initial_update_topology(initial_urls, self.application_id)
                //     .await;
                // if let Err(e) = result {
                //     tracing::error!(
                //         "An error occurred while running the initial topology update. Caused by: {:?}",
                //         e
                //     );
                // }
                unimplemented!();
            }
            RequestExecutorMessage::UpdateTopology => {
                todo!();
            }
            RequestExecutorMessage::TopologyUpdated { topology } => {
                todo!();
            }
        }
    }

    async fn wait_for_initial_topology(
        &self,
        initial_urls: Vec<Url>,
        application_id: Uuid,
    ) -> Result<(), Vec<(Url, RequestExecutorError)>> {
        Ok(())
    }

    async fn get_topology_from_node(
        parameters: UpdateTopologyParameters,
    ) -> Result<(), RequestExecutorError> {
        todo!()
    }

    fn get_topology_nodes(&self) -> Option<HashSet<ServerNode>> {
        if let Some(topology) = self.get_topology() {
            Some(topology.nodes)
        } else {
            None
        }
    }

    fn get_topology(&self) -> Option<DatabaseTopology> {
        self.topology.clone()
    }

    /// Returns the fastest node available if one exists.
    fn get_fastest_node(&self) -> Option<ServerNode> {
        // TODO: actually return the fastest node
        // For now just return the first node
        self.get_preferred_node()
    }

    /// Returns a specific node for the given session id.
    fn get_node_by_session_id(&self, session_id: i32) -> Option<ServerNode> {
        // TODO: actually return the session_id node.
        // For now just return preferred node
        self.get_preferred_node()
    }

    /// Returns the currently preferred node.
    /// Right now this looks for the first node with 0 failures and returns it.
    /// On the off chance all nodes have failures, it returns a random node.
    fn get_preferred_node(&self) -> Option<ServerNode> {
        let x = self.topology.as_ref().and_then(|topology| {
            topology
                .node_failures
                .iter()
                .find(|(_, count)| **count == 0)
                .and_then(|(node, _)| Some(node.clone()))
        });

        if x.is_some() {
            return x;
        }

        // If all nodes are marked with failures, just select one at random. This may still
        // be `None` if the topology is empty.
        // NOTE: JVM version rotates through an index but this lib uses a HashMap to store
        // the nodes and ordering is irrelevant, so a random choice makes more sense.
        self.select_random_node()
    }

    /// Returns the requested node by node tag.
    fn get_requested_node(&self, tag: String) -> Option<ServerNode> {
        // TODO: actually return the fastest node
        // For now just return the first node
        self.get_preferred_node()
    }

    /// Returns a random node if all are faulted.
    fn select_random_node(&self) -> Option<ServerNode> {
        if let Some(topology) = &self.topology {
            let mut rng = thread_rng();
            topology.nodes.iter().choose(&mut rng).cloned()
        } else {
            None
        }
    }
}

#[instrument(level = "debug")]
async fn initial_update_topology(
    initial_urls: Vec<Url>,
    database: String,
    application_id: Uuid,
) -> Result<DatabaseTopology, Vec<(Url, RequestExecutorError)>> {
    // Note: Java client implementation validates URL strings here.
    // This rust library does not because the strings are validated by the DocumentStoreBuilder
    // and are already valid `reqwest::Url`s before they arrive at this point.

    let mut server_errors = Vec::new();

    for url in initial_urls.iter() {
        let server_node = ServerNode::new(url.clone(), database.clone());
        let update_parameters = UpdateTopologyParameters {
            server_node: server_node.clone(),
            timeout_in_ms: i32::MAX, //TODO: Is this necessary? I believe it has something to do with a tcp timeout bug, but maybe only in java or C#
            force_update: false,
            application_id,
        };

        let x = update_topology_async(update_parameters).await;

        match x {
            Ok(result) => {
                // Yay, the topology is updated, return early
                tracing::info!("Initial topology update complete");
                return Ok(result);
            }
            Err(e) => {
                server_errors.push((url.clone(), e));
            }
        }
        // No timer initialized here like JVM client. Actor runner handles the timer.
    }
    // If this point is reached, none of the provided URLs succeeded in providing a topology
    // for one reason or another.

    // Create a set of [`ServerNode`]s from the initial urls. Hope these
    // servers are actually online and listening.
    let nodes = initial_urls
        .iter()
        .map(|url| {
            let mut server_node = ServerNode::new(url.clone(), database.clone());

            server_node.cluster_tag = "!".to_string();
            server_node
        })
        .collect::<HashSet<ServerNode>>();

    // Create a new topology from manufactured one above.
    let topology = DatabaseTopology {
        nodes,
        ..Default::default()
    };

    // Ensure the user did not somehow pass an empty list of URLs.
    if !initial_urls.is_empty() {
        // No timer initialized here like JVM client. Actor runner handles the timer.
        return Ok(topology);
    }

    // Return the errors to the caller to deal with
    let server_errors = server_errors
        .into_iter()
        .map(|e| (e.0, e.1))
        .collect::<Vec<_>>();
    Err(server_errors)
}

async fn update_topology_async(
    parameters: UpdateTopologyParameters,
) -> Result<DatabaseTopology, RequestExecutorError> {
    todo!();
}

struct TopologyUpdateResult {
    topology: DatabaseTopology,
}

struct UpdateTopologyParameters {
    server_node: ServerNode,
    timeout_in_ms: i32,
    force_update: bool,
    application_id: Uuid,
}

#[instrument(level = "debug", skip(client_identity))]
async fn send_raven_command_request_to_server(
    client_identity: Option<Identity>,
    dns_overrides: DnsOverrides,
    proxy_address: Option<String>,
    raven_command: RavenCommand,
    topology_etag: u64,
) -> anyhow::Result<reqwest::Response> {
    let mut client = reqwest::Client::builder();

    if let Some(identity) = client_identity.clone() {
        client = client.identity(identity).use_rustls_tls();
    }

    // Convert Option<HashMap<String, IpAddr>> into HashMap<String,SocketAddr>
    let overrides = dns_overrides
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

#[instrument(level = "debug", name = "Running Document Store Actor", skip(actor))]
pub async fn run_request_executor_actor(mut actor: RequestExecutorActor) {
    // Run a 5 minute timer to send topology update requests to the actor
    let mut topology_update_timer = tokio::time::interval(tokio::time::Duration::from_secs(60 * 5));
    // Run a 1 minute timer to request database topology updates
    let mut topology_update_timer_1min =
        tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        tokio::select! {
            // 5 minute timer
            _ = topology_update_timer.tick() => {
                tracing::debug!("Updating topology via timer.");
                let _= actor.sender_internal.send(RequestExecutorMessage::UpdateTopology).await;
            }
            // 1 minute timer
            _ = topology_update_timer_1min.tick() => {
                tracing::debug!("Updating topology via 1 minute timer.");
                let _ = actor.sender_internal.send(RequestExecutorMessage::UpdateTopology).await;
            }
            // Messages from the handle
            external_message = actor.receiver.recv() => {
                let msg = match external_message {
                    Some(msg) => msg,
                    None=> break,
                };
                actor.handle_message(msg).await;
            },
            // Messages from sub-tasks of this actor
            Some(internal_message) = actor.receiver_internal.recv() => {
                actor.handle_message(internal_message).await;
            }
        }
    }
}
