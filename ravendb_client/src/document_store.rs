use std::{collections::HashMap, fs::File, io::Read};

use anyhow::Context;
use rand::seq::IteratorRandom;
use reqwest::Identity;
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;
use url::Url;

use crate::{
    cluster_topology::{ClusterTopologyInfo, ClusterTopology},
    error_chain_fmt,
    raven_command::RavenCommand,
    DocumentSession,
};

#[derive(Debug)]
pub struct DocumentStoreBuilder {
    database_name: Option<String>,
    document_store_urls: Vec<String>,
    client_certificate_path: Option<String>,
}

impl DocumentStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_urls<T>(mut self, urls: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        for u in urls {
            self.document_store_urls.push(u.as_ref().to_string());
        }
        self
    }

    pub fn set_client_certificate(mut self, certificate_path: &str) -> Self {
        self.client_certificate_path = Some(certificate_path.to_string());
        self
    }

    pub fn set_database_name(mut self, database_name: &str) -> Self {
        self.database_name = Some(database_name.to_string());
        self
    }

    /// Initializes a new [`DocumentStoreActor`] and retuns a handle to it.
    ///
    /// Each call to this will create a new [`DocumentStoreActor`] and return a new handle to it.
    /// It is not recommended to create more that one per database cluster. This function is allowed
    /// to be called more than once to the builder can act as a template after being set up once.
    pub fn build(&self) -> anyhow::Result<DocumentStore> {
        // Ensure DocumentStore URLs are valid and there is at least one
        assert!(!self.document_store_urls.is_empty());

        // Validate URLS
        let initial_node_list = validate_urls(
            self.document_store_urls.as_slice(),
            self.client_certificate_path.is_some(),
        )?;

        let topology_info = ClusterTopologyInfo {
            topology: ClusterTopology {
                all_nodes: initial_node_list,
                ..Default::default()
            },
            ..Default::default()
        };

        let identity = match &self.client_certificate_path {
            Some(certpath) => {
                // Open and validate certificate, and create an identity from it
                let mut buf = Vec::new();
                File::open(certpath)?.read_to_end(&mut buf)?;
                let id = reqwest::Identity::from_pem(&buf)?;
                Some(id)
            }
            None => None,
        };

        // Create an initial configuration for the DocumentStoreActor
        let initial_config = DocumentStoreInitialConfiguration {
            //async_document_id_generator: self.async_document_id_generator.clone(),
            database_name: self.database_name.clone(),
            cluster_topology: topology_info,
            client_identity: identity,
        };

        Ok(DocumentStore::new(initial_config))
    }
}

#[allow(clippy::derivable_impls)] //TODO: Remove this allow when ready
impl Default for DocumentStoreBuilder {
    fn default() -> Self {
        // TODO: Create a default async id generator in the Default implementation

        Self {
            //async_document_id_generator: Box::new(AsyncMultiDatabaseHiLoIdGenerator::default()),
            database_name: None,
            document_store_urls: Vec::new(),
            client_certificate_path: None,
        }
    }
}

/**
This a handle to the actor.

Only one DocumentStoreActor should exist per database cluster when possible to reduce resource
usage. Cloning this handle is very cheap and will not instantiate a new actor in the background.
It is recommended to clone this handle to each component that needs to talk to the DocumentStoreActor.
When the last handle goes out of scope and it dropped, the backing actor will also be dropped.

TODO: Uncomment this example after build function is completed
```rust
// # use tokio_test;
// # tokio_test::block_on(async {
// use ravendb_client::DocumentStore;
// use ravendb_client::DocumentStoreBuilder;

// let document_store: DocumentStore = DocumentStoreBuilder::new().build();
// println!("DEBUG: {:?}",document_store);
// # })
```
*/
#[derive(Clone, Debug)]
pub struct DocumentStore {
    sender: mpsc::Sender<DocumentStoreMessage>,
}

impl DocumentStore {
    pub fn builder() -> DocumentStoreBuilder {
        DocumentStoreBuilder::default()
    }

    // TODO: make this documentstore handle into a builder, or create a builder to set defaults and return the handle
    // after creating the actor. Which is better?
    // This is pub(crate) so only the builder can crank it out
    pub(crate) fn new(initial_config: DocumentStoreInitialConfiguration) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = DocumentStoreActor::new(receiver, initial_config);
        tokio::spawn(run_document_store_actor(actor));

        Self { sender }
    }

    #[instrument(name = "ACTOR HANDLE - Execute Raven Command", skip(self))]
    pub async fn execute_raven_command(
        &self,
        raven_command: RavenCommand,
    ) -> Result<reqwest::Response, anyhow::Error> {
        tracing::debug!("Creating oneshot channel");
        let (tx, rx) = oneshot::channel();
        tracing::debug!("Sending message to actor");
        let _ = self
            .sender
            .send(DocumentStoreMessage::ExecuteRavenCommand {
                raven_command,
                respond_to: tx,
            })
            .await;
        tracing::debug!("Waiting for oneshot to return");
        rx.await?.context("DocumentStoreActor task has been killed")
    }

    #[instrument(name = "ACTOR HANDLE - Get Server Address", skip(self))]
    pub async fn get_server_address(&self) -> anyhow::Result<Url> {
        tracing::debug!("Getting a server address");
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetServerAddress { respond_to: tx })
            .await;
        rx.await?.context("DocumentStoreActor task has been killed")
    }

    pub async fn open_session(&self) -> Result<DocumentSession, DocumentStoreError> {
        let session = DocumentSession::new(self.clone());
        Ok(session)
    }
}

struct DocumentStoreActor {
    receiver: mpsc::Receiver<DocumentStoreMessage>,
    //_async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    client_identity: Option<reqwest::Identity>,
    _conventions: Option<Conventions>,
    _database_name: Option<String>,
    _trust_store: Option<CertificatePlaceholder>,
    topology_info: ClusterTopologyInfo,
}
impl DocumentStoreActor {
    fn new(
        receiver: mpsc::Receiver<DocumentStoreMessage>,
        initial_config: DocumentStoreInitialConfiguration,
    ) -> Self {
        Self {
            receiver,
            //_async_document_id_generator: initial_config.async_document_id_generator,
            topology_info: initial_config.cluster_topology,
            _conventions: Default::default(),
            _database_name: initial_config.database_name,
            client_identity: initial_config.client_identity,
            _trust_store: Some(CertificatePlaceholder),
        }
    }

    #[instrument(name = "DocumentStore Actor - Handle Message", skip(self))]
    async fn handle_message(&mut self, msg: DocumentStoreMessage) {
        //TODO: Move all these handler bodies into functions in their own module or modules and call them
        // to avoid massive bloat in this match statement
        match msg {
            DocumentStoreMessage::ExecuteRavenCommand {
                raven_command,
                respond_to,
            } => {
                let client_identity = self.client_identity.clone();
                tokio::spawn(async move {
                    let result =
                        DocumentStoreActor::execute_raven_command(client_identity, raven_command)
                            .await;
                    let _ = respond_to.send(result);
                });
            }
            DocumentStoreMessage::GetServerAddress { respond_to } => {
                let result = self.get_server_address().await;
                let _ = respond_to.send(result);
            }
        }
    }

    #[instrument(
        name = "DocumentStore Actor - Execute Raven Command",
        skip(client_identity)
    )]
    async fn execute_raven_command(
        client_identity: Option<Identity>,
        raven_command: RavenCommand,
    ) -> anyhow::Result<reqwest::Response> {
        let mut client = reqwest::Client::builder();

        if let Some(identity) = client_identity {
            client = client.identity(identity).use_rustls_tls();
        }

        let client = client.build()?;
        let response = client.execute(raven_command.get_http_request()?).await?;

        Ok(response)
    }

    #[instrument(name = "DocumentStore Actor - Get Server Address", skip(self))]
    async fn get_server_address(&self) -> anyhow::Result<Url> {
        self.topology_info
            .topology
            .all_nodes
            .values()
            .choose(&mut rand::thread_rng())
            .context("Urls list is empty")
            .cloned()
    }
}

#[instrument(name = "Run Document Store Actor", skip(actor))]
async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Debug)]
enum DocumentStoreMessage {
    //TODO: Consider having all of these just return the json and let the handle do
    // the data crunching and deserialization to free up the actor's message queue faster
    // -- may not be necessary with async but look into it
    /// Executes the provided [`RavenCommand`].
    ExecuteRavenCommand {
        raven_command: RavenCommand,
        // TODO: Change this to a DocumentStoreError or maybe a RavenError
        respond_to: oneshot::Sender<Result<reqwest::Response, anyhow::Error>>,
    },
    GetServerAddress {
        respond_to: oneshot::Sender<Result<Url, anyhow::Error>>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum DocumentStoreState {
    /// [`DocumentStore`] was initialized but has since been closed.
    Closed,

    /// [`DocumentStore`] is initialized.
    Initialized,

    /// [`DocumentStore`] has not yet been initialized.
    Unitilialized,
}

/// Requests to initialize.
pub(crate) struct DocumentStoreInitialConfiguration {
    //async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    database_name: Option<String>,
    cluster_topology: ClusterTopologyInfo,
    client_identity: Option<reqwest::Identity>,
}

// Placeholders below
#[derive(Debug)]
pub struct Conventions;
pub struct CertificatePlaceholder;

pub struct DatabaseChanges;
pub struct DatabaseChangesBuilder;

#[derive(Debug)]
pub struct DocumentSubscription;

#[derive(thiserror::Error)]
pub enum DocumentStoreError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl std::fmt::Debug for DocumentStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Converts the provided URL strings to a [`Vec`] of [`Url`], ensuring they are a valid format.
///
/// Also ensures all provided URL strings use the same schema: either https or http, but never both within the
/// list.
fn validate_urls<T>(urls: &[T], require_https: bool) -> anyhow::Result<HashMap<String, Url>>
where
    T: AsRef<str>,
{
    //let mut clean_urls = Vec::new();

    //TODO: Check URLs are valid
    //TODO: Check all URLs are either http OR https, no mixing

    let clean_urls = urls
        .iter()
        .flat_map(|url| -> anyhow::Result<Url> { Ok(Url::parse(url.as_ref())?) })
        .map(|url| (url.to_string(), url))
        .collect::<HashMap<_, _>>();

    let desired_scheme = if require_https { "https" } else { "http" };

    for url in clean_urls.values().collect::<Vec<_>>() {
        if url.scheme() != desired_scheme {
            return Err(anyhow::anyhow!("Url does not have correct scheme: {}", url));
        }
    }

    Ok(clean_urls)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::collections::HashMap;

    use url::Url;

    use crate::DocumentStoreBuilder;

    use super::validate_urls;

    #[test]
    fn validate_urls_returns_correct_HashMap_for_http_strings() {
        // Arrange
        let mut baseline_urls = HashMap::<String, Url>::new();
        baseline_urls.insert(
            "http://starwars.com/".to_string(),
            Url::parse("http://starwars.com").unwrap(),
        );
        baseline_urls.insert(
            "http://google.com/".to_string(),
            Url::parse("http://google.com").unwrap(),
        );

        let urls = vec!["http://starwars.com", "http://google.com"];

        // Act
        let result = validate_urls(urls.as_slice(), false).unwrap();
        // Assert
        assert_eq!(result, baseline_urls);
    }

    #[test]
    fn validate_urls_returns_correct_HashMap_for_https_strings() {
        // Arrange
        let mut baseline_urls = HashMap::<String, Url>::new();
        baseline_urls.insert(
            "https://starwars.com/".to_string(),
            Url::parse("https://starwars.com").unwrap(),
        );
        baseline_urls.insert(
            "https://google.com/".to_string(),
            Url::parse("https://google.com").unwrap(),
        );

        let urls = vec!["https://starwars.com", "https://google.com"];

        // Act
        let result = validate_urls(urls.as_slice(), true).unwrap();

        // Assert
        assert_eq!(result, baseline_urls);
    }

    #[test]
    fn validate_urls_fails_for_mixed_http_and_https_strings() {
        // Arrange
        let urls = vec!["https://starwars.com", "http://google.com"];

        // Assert
        assert!(validate_urls(urls.as_slice(), true).is_err());
        assert!(validate_urls(urls.as_slice(), false).is_err());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_succeeds_for_valid_configuration() {
        // Arrange
        let urls = ["https://localhost:8080"];

        let document_store = DocumentStoreBuilder::new()
            .set_client_certificate("../ravendb-client_dev_cert.pem")
            .set_urls(&urls)
            .build();

        // Assert
        assert!(document_store.is_ok());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_fails_for_invalid_pem() {
        // Arrange
        let urls = ["https://localhost:8080"];

        let document_store = DocumentStoreBuilder::new()
            // README.md is not a valid PEM file
            .set_client_certificate("../README.md")
            .set_urls(&urls)
            .build();

        // Assert
        assert!(document_store.is_err());
    }
}
