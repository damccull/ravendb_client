use reqwest::{Identity, Url};
use tokio::sync::mpsc;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    document_conventions::DocumentConventions, server_node::ServerNode, topology::Topology,
};

use super::{RequestExecutorError, RequestExecutorMessage};

pub struct RequestExecutorActor {
    /// Allows the server to warn if [`DocumentStore`] is being recreated too many times
    /// instead of once per application. RequestExecutor should be cached and reused, so
    /// this shouldn't change after initialization.
    application_id: Uuid,
    conventions: DocumentConventions,
    database: String,
    identity: Identity,
    last_known_urls: Vec<Url>,
    receiver: mpsc::Receiver<RequestExecutorMessage>,
    reqwest_client: reqwest::Client,
    topology_taken_from_node: Option<ServerNode>, // TODO: Find a better name. This is the node our topology came from.
}

impl RequestExecutorActor {
    pub(crate) fn new(
        receiver: mpsc::Receiver<RequestExecutorMessage>,
        database: String,
        identity: Identity,
        conventions: DocumentConventions,
    ) -> Self {
        // Reqwest client maintains an internal connection pool. Reuse it so long as this
        // RequestExecutor lives.
        let reqwest_client = reqwest::Client::new();

        // if let Some(identity) = client_identity.clone() {
        //                 client = client.identity(identity).use_rustls_tls();
        //             }

        //TODO: Kick off first topology update

        Self {
            application_id: Uuid::new_v4(),
            conventions,
            database,
            identity,
            last_known_urls: Vec::default(),
            receiver,
            reqwest_client,
            topology_taken_from_node: Option::default(),
        }
    }
    async fn handle_message(&self, msg: RequestExecutorMessage) {
        match msg {
            RequestExecutorMessage::ExecuteRequest {
                respond_to,
                request,
            } => todo!(),
            RequestExecutorMessage::FirstTopologyUpdate { initial_urls } => {
                let result = self.first_topology_update(initial_urls).await;
                if let Err(e) = result {
                    tracing::error!(
                        "An error occurred while running the first topology update. Caused by: {}",
                        e
                    );
                }
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    async fn first_topology_update(
        &self,
        initial_urls: Vec<Url>,
        application_id: Uuid,
    ) -> Result<(), Vec<(Url, RequestExecutorError)>> {
        // Note: Java client implementation validates URL strings here.
        // This rust library does not because the strings are validated by the DocumentStoreBuilder
        // and are already valid `reqwest::Url`s before they arrive at this point.

        let mut server_errors = Vec::new();

        for url in &initial_urls {
            let server_node = ServerNode::new(url.clone(), self.database.clone());
            let update_parameters = UpdateTopologyParameters {
                server_node,
                timeout_in_ms: i32::MAX,
                force_update: false,
                application_id,
            };

            let x = RequestExecutorActor::update_topology(update_parameters).await;

            match x {
                Ok(_) => {
                    // Yay, the topology is updated, return early
                    tracing::info!("Initial topology update complete");
                    self.topology_taken_from_node = Some(server_node);
                    return Ok(());
                }
                Err(e) => {
                    server_errors.push((url, e));
                }
            }

            // if x.is_err() {
            //     server_errors.push((url, x.unwrap_err()));
            // } else {
            //     // Yay, the topology is updated, return early
            //     tracing::info!("Initial topology update complete");
            //     self.topology_taken_from_node = Some(server_node);
            //     return Ok(());
            // }
            // No timer initialized here like JVM client. Actor runner handles the timer.
        }
        // If this point is reached, none of the provided URLs succeeded in providing a topology
        // for one reason or another. At this point, try to get a topology from the current
        // NodeSelector, if one exists.

        let mut nodes = self.get_topology_nodes();

        // If no list of nodes came back from the current NodeSelector, either because it doesn't
        // exist or its topology is empty/None, create a topology from the initial urls. Hope these
        // servers are actually online and listening.
        if nodes.is_none() {
            nodes = Some(
                initial_urls
                    .into_iter()
                    .map(|url| {
                        let server_node = ServerNode::new(url.clone(), self.database.clone());
                        server_node.set_cluster_tag("!".to_string());
                        server_node
                    })
                    .collect::<Vec<_>>(),
            );
        }

        // Create a new topology from the NodeSelector topology, or from the manufactured one above.
        let topology = Topology {
            nodes,
            ..Default::default()
        };

        // Create a new NodeSelector from the newly created topology.
        self.node_selector = NodeSelector::new(topology);

        // Ensure the user did not somehow pass an empty list of URLs.
        if initial_urls.len() > 0 {
            // No timer initialized here like JVM client. Actor runner handles the timer.
            return Ok(());
        }

        // Save the initial urls in case they're needed for something later.
        // TODO: Fix the above comment when you figure out what this is for.
        self.last_known_urls = initial_urls;

        // Return the errors to the caller to deal with
        let server_errors = server_errors.iter().map(|e| (e.0, e.1)).collect::<Vec<_>>();
        Err(server_errors)
    }

    async fn update_topology(
        parameters: UpdateTopologyParameters,
    ) -> Result<(), RequestExecutorError> {
        todo!()
    }

    fn get_topology_nodes(&self) -> Option<Vec<ServerNode>> {
        self.get_topology().and_then(|t| t.nodes)
    }

    fn get_topology(&self) -> Option<Topology> {
        todo!()
    }
}

struct UpdateTopologyParameters {
    server_node: ServerNode,
    timeout_in_ms: i32,
    force_update: bool,
    application_id: Uuid,
}

// #[instrument(level = "debug", skip(client_identity))]
//     async fn send_raven_command_request_to_server(
//         client_identity: Option<Identity>,
//         dns_overrides: Option<DnsOverrides>,
//         proxy_address: Option<String>,
//         raven_command: RavenCommand,
//         topology_etag: i64,
//     ) -> anyhow::Result<reqwest::Response> {
//         let mut client = reqwest::Client::builder();

//         if let Some(identity) = client_identity.clone() {
//             client = client.identity(identity).use_rustls_tls();
//         }

//         // Convert Option<HashMap<String, IpAddr>> into HashMap<String,SocketAddr>
//         let overrides = dns_overrides
//             .unwrap_or_default()
//             .into_iter()
//             .map(|(k, v)| (k, SocketAddr::new(v, 0)))
//             .collect::<HashMap<String, SocketAddr>>();

//         for (domain, address) in overrides {
//             tracing::trace!(
//                 "Adding `{}->{}` to dns overrides for this request.",
//                 domain,
//                 address
//             );
//             client = client.resolve(domain.as_str(), address);
//         }

//         if let Some(proxy) = proxy_address {
//             tracing::trace!("Proxy set to `{}`", &proxy);
//             client = client.proxy(reqwest::Proxy::http(proxy)?);
//         } else {
//             tracing::trace!("No proxy defined. Using system settings.");
//         }

//         let client = client.build()?;

//         let mut request = raven_command.get_http_request()?;
//         let headerval = HeaderValue::from_str(topology_etag.to_string().as_str())?;
//         request.headers_mut().append("Topology-Etag", headerval);
//         tracing::trace!("Request Headers: {:#?}", &request.headers());
//         let response = client.execute(request).await?;

//         Ok(response)
//     }

#[instrument(level = "debug", name = "Running Document Store Actor", skip(actor))]
pub async fn run_request_executor_actor(mut actor: RequestExecutorActor) {
    //TODO: Run a 5 minute timer to send topology update requests to the actor
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}
