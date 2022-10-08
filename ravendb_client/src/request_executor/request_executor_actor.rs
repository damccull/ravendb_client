use reqwest::{Identity, Url};
use tokio::sync::mpsc;
use tracing::{instrument, Id};

use crate::{document_conventions::DocumentConventions, server_node::ServerNode};

use super::RequestExecutorMessage;

pub struct RequestExecutorActor {
    conventions: DocumentConventions,
    database: String,
    identity: Identity,
    receiver: mpsc::Receiver<RequestExecutorMessage>,
    reqwest_client: reqwest::Client,
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
            conventions,
            database,
            identity,
            receiver,
            reqwest_client,
        }
    }
    async fn handle_message(&self, msg: RequestExecutorMessage) {
        match msg {
            RequestExecutorMessage::ExecuteRequest {
                respond_to,
                request,
            } => todo!(),
            RequestExecutorMessage::FirstTopologyUpdate { initial_urls } => todo!(),
        }
    }

    async fn first_topology_update(&self, initial_urls: Vec<Url>) {
        // Note: Java client implementation validates URL strings here.
        // This rust library does not because the strings are validated by the DocumentStoreBuilder
        // and are already valid `reqwest::Url`s before they arrive at this point.

        for url in initial_urls {
            let server_node = ServerNode::new(url, self.database.clone());
        }
    }

    async fn update_topology(parameters: UpdateTopologyParameters) {
        
    }
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
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}
