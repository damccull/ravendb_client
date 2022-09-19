use std::{fs::File, io::Read};

use anyhow::Context;
use rand::seq::SliceRandom;
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;
use url::Url;

use crate::{error_chain_fmt, raven_command::RavenCommand, DocumentSession};

#[derive(Debug)]
pub struct DocumentStoreBuilder {
    database_name: Option<String>,
    document_store_urls: Vec<String>,
    client_certificate_path: String,
    require_https: bool,
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
        self.client_certificate_path = certificate_path.to_string();
        self
    }

    pub fn set_database_name(mut self, database_name: &str) -> Self {
        self.database_name = Some(database_name.to_string());
        self
    }

    pub fn require_https(mut self) -> DocumentStoreBuilder {
        self.require_https = true;
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
        // Ensure the only valid combination of https and certificate path is TRUE and a valid path
        if self.require_https {
            assert!(!self.client_certificate_path.is_empty());
        }
        if !self.require_https {
            assert!(self.client_certificate_path.is_empty());
        }

        // Validate URLS
        let clean_urls = validate_urls(self.document_store_urls.as_slice(), self.require_https)?;

        let identity = if self.require_https {
            // Open and validate certificate, and create an identity from it
            let mut buf = Vec::new();
            File::open(&self.client_certificate_path)?.read_to_end(&mut buf)?;
            let identity = reqwest::Identity::from_pem(&buf)?;
            Some(identity)
        } else {
            None
        };

        // Create an initial configuration for the DocumentStoreActor
        let initial_config = DocumentStoreInitialConfiguration {
            //async_document_id_generator: self.async_document_id_generator.clone(),
            database_name: self.database_name.clone(),
            cluster_urls: clean_urls,
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
            client_certificate_path: String::default(),
            require_https: false,
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
    urls: Vec<Url>,
}
impl DocumentStoreActor {
    fn new(
        receiver: mpsc::Receiver<DocumentStoreMessage>,
        initial_config: DocumentStoreInitialConfiguration,
    ) -> Self {
        Self {
            receiver,
            //_async_document_id_generator: initial_config.async_document_id_generator,
            urls: initial_config.cluster_urls,
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
                //TODO: Convert this to a tokio spawn and send the respond_to to the helper fn instead
                // of responding here. This will allow the system to spin of a bunch of simultaneous
                // RavenCommands.
                let result = self.execute_raven_command(raven_command).await;
                let _ = respond_to.send(result);
            }
            DocumentStoreMessage::GetServerAddress { respond_to } => {
                let result = self.get_server_address().await;
                let _ = respond_to.send(result);
            }
        }
    }

    #[instrument(name = "DocumentStore Actor - Execute Raven Command", skip(self))]
    async fn execute_raven_command(
        &self,
        raven_command: RavenCommand,
    ) -> anyhow::Result<reqwest::Response> {
        let mut client = reqwest::Client::builder();

        if let Some(identity) = &self.client_identity {
            client = client.identity(identity.clone()).use_rustls_tls();
        }

        let client = client.build()?;
        let response = client.execute(raven_command.get_http_request()?).await?;

        Ok(response)
    }

    #[instrument(name = "DocumentStore Actor - Get Server Address", skip(self))]
    async fn get_server_address(&self) -> anyhow::Result<Url> {
        self.urls
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow::anyhow!("Urls list is empty"))
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
    cluster_urls: Vec<Url>,
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
fn validate_urls<T>(urls: &[T], require_https: bool) -> anyhow::Result<Vec<Url>>
where
    T: AsRef<str>,
{
    //let mut clean_urls = Vec::new();

    //TODO: Check URLs are valid
    //TODO: Check all URLs are either http OR https, no mixing

    let clean_urls = urls
        .iter()
        .map(|url| -> anyhow::Result<Url> {
            let url: Url = match Url::parse(url.as_ref()) {
                Ok(u) => u,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Invalid URL: {}; container error: {}",
                        url.as_ref(),
                        e
                    ));
                }
            };
            Ok(url)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let desired_scheme = if require_https { "https" } else { "http" };

    for url in &clean_urls {
        if url.scheme() != desired_scheme {
            return Err(anyhow::anyhow!("Url does not have correct scheme: {}", url));
        }
    }

    Ok(clean_urls)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use url::Url;

    use crate::DocumentStoreBuilder;

    use super::validate_urls;

    #[test]
    fn validate_urls_returns_vec_of_URL_for_http_strings() {
        let baseline_urls = vec![
            Url::parse("http://starwars.com").unwrap(),
            Url::parse("http://google.com").unwrap(),
        ];
        let urls = vec!["http://starwars.com", "http://google.com"];

        assert_eq!(
            validate_urls(urls.as_slice(), false).unwrap(),
            baseline_urls
        );
    }

    #[test]
    fn validate_urls_returns_vec_of_URL_for_https_strings() {
        let baseline_urls = vec![
            Url::parse("https://starwars.com").unwrap(),
            Url::parse("https://google.com").unwrap(),
        ];
        let urls = vec!["https://starwars.com", "https://google.com"];

        assert_eq!(validate_urls(urls.as_slice(), true).unwrap(), baseline_urls);
    }

    #[test]
    fn validate_urls_fails_for_mixed_http_and_https_strings() {
        let urls = vec!["https://starwars.com", "http://google.com"];

        assert!(validate_urls(urls.as_slice(), true).is_err());
        assert!(validate_urls(urls.as_slice(), false).is_err());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_succeeds_for_valid_configuration() {
        let urls = ["https://localhost:8080"];

        let document_store = DocumentStoreBuilder::new()
            .set_client_certificate("../free.damccull.client.certificate.pem")
            .require_https()
            .set_urls(&urls)
            .build();

        assert!(document_store.is_ok());
    }

    #[tokio::test]
    #[should_panic]
    async fn documentstorebuilder_build_fails_if_requires_https_but_no_certificate() {
        let urls = ["https://localhost:8080"];

        let _document_store = DocumentStoreBuilder::new()
            .require_https()
            .set_urls(&urls)
            .build();
    }

    #[tokio::test]
    #[should_panic]
    async fn documentstorebuilder_build_fails_if_certificate_supplied_but_https_not_required() {
        let urls = ["http://localhost:8080"];

        let _document_store = DocumentStoreBuilder::new()
            .set_urls(&urls)
            .set_client_certificate("../free.damccull.client.certificate.pem")
            .build();
    }
}
