use std::{fs::File, io::Read};

use tokio::sync::{broadcast, mpsc, oneshot};
use url::Url;

use crate::{
    async_multi_database_hi_lo_id_generator::{
        AsyncDocumentIdGenerator, AsyncMultiDatabaseHiLoIdGenerator,
    },
    events::{ConversionEvents, CrudEvents, RequestEvents, SessionEvents},
    DocumentSession,
};

#[derive(Debug)]
pub struct DocumentStoreBuilder {
    async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>, // TODO: Change this to a trait impl later
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

    pub fn set_async_document_id_generator(
        mut self,
        generator: Box<dyn AsyncDocumentIdGenerator>,
    ) -> Self {
        self.async_document_id_generator = generator;
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
        // TODO: Assert the configuration supplied is valid
        // Ensure DocumentStore URLs are valid and there is at least one
        assert!(!self.document_store_urls.is_empty());

        // Validate URLS
        // TODO: Check if https is required and use the preference
        let clean_urls = validate_urls(self.document_store_urls.as_slice(), self.require_https)?;

        // Validate certificate has a private key
        let mut buf = Vec::new();
        File::open(&self.client_certificate_path)?.read_to_end(&mut buf)?;
        let identity = reqwest::Identity::from_pem(&buf)?;

        // Create an initial configuration for the DocumentStoreActor
        let initial_config = DocumentStoreInitialConfiguration {
            async_document_id_generator: self.async_document_id_generator.clone(),
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
            async_document_id_generator: Box::new(AsyncMultiDatabaseHiLoIdGenerator::default()),
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

    pub async fn aggressively_cache(&self) -> Result<(), DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::AggressivelyCache { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn close(&self) -> Result<(), DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::Close { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_conventions(&self) -> Result<DocumentConventions, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetConventions { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_database(&self) -> Result<Option<String>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetDatabase { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_document_store_identifier(&self) -> Result<String, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetDocumentStoreIdentifier { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_document_store_state(&self) -> DocumentStoreState {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetDocumentStoreState { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_receiver_for_conversion_events(
        &self,
    ) -> Result<broadcast::Receiver<ConversionEvents>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetReceiverForConversionEvents { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_receiver_for_crud_events(
        &self,
    ) -> Result<broadcast::Receiver<CrudEvents>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetReceiverForCrudEvents { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_receiver_for_request_events(
        &self,
    ) -> Result<broadcast::Receiver<RequestEvents>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetReceiverForRequestEvents { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_receiver_for_session_events(
        &self,
    ) -> Result<broadcast::Receiver<SessionEvents>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetReceiverForSessionEvents { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_urls(&self) -> Result<Vec<Url>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetUrls { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<DocumentSubscription>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetSubscriptions { respond_to: tx });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn open_session(&self) -> Result<DocumentSession, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::OpenSession { respond_to: tx })
            .await;
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn set_conventions(
        &self,
        conventions: DocumentConventions,
    ) -> Result<DocumentConventions, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(DocumentStoreMessage::SetConventions {
            respond_to: tx,
            conventions,
        });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn set_database(&self, database: String) -> Result<String, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(DocumentStoreMessage::SetDatabase {
            respond_to: tx,
            database,
        });
        rx.await.expect("DocumentStoreActor task has been killed")
    }

    pub async fn set_urls(&self, urls: Vec<Url>) -> Result<Vec<Url>, DocumentStoreError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(DocumentStoreMessage::SetUrls {
            respond_to: tx,
            urls,
        });
        rx.await.expect("DocumentStoreActor task has been killed")
    }
}

struct DocumentStoreActor {
    receiver: mpsc::Receiver<DocumentStoreMessage>,

    document_store_state: DocumentStoreState,

    conversion_events_sender: broadcast::Sender<ConversionEvents>,
    crud_events_sender: broadcast::Sender<CrudEvents>,
    request_events_sender: broadcast::Sender<RequestEvents>,
    session_events_sender: broadcast::Sender<SessionEvents>,

    _async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    _client_identity: reqwest::Identity,
    _conventions: Option<Conventions>,
    _database: Option<String>,
    _trust_store: Option<CertificatePlaceholder>,
    _urls: Vec<Url>,
}
impl DocumentStoreActor {
    fn new(
        receiver: mpsc::Receiver<DocumentStoreMessage>,
        initial_config: DocumentStoreInitialConfiguration,
    ) -> Self {
        let (crud_sender, _) = broadcast::channel(100);
        let (request_sender, _) = broadcast::channel(100);
        let (conversion_sender, _) = broadcast::channel(100);
        let (session_sender, _) = broadcast::channel(100);
        Self {
            receiver,
            document_store_state: DocumentStoreState::Unitilialized,
            crud_events_sender: crud_sender,
            request_events_sender: request_sender,
            conversion_events_sender: conversion_sender,
            session_events_sender: session_sender,
            _async_document_id_generator: initial_config.async_document_id_generator,
            _urls: initial_config.cluster_urls,
            _conventions: Default::default(),
            _database: Default::default(),
            _client_identity: initial_config.client_identity,
            _trust_store: Some(CertificatePlaceholder),
        }
    }

    fn handle_message(&mut self, msg: DocumentStoreMessage) {
        //TODO: Move all these handler boies into functions in their own module or modules and call them
        // to avoid massive bloat in this match statement
        match msg {
            DocumentStoreMessage::Close { respond_to } => {
                let _ = respond_to.send(Ok(()));
                todo!();
            }
            DocumentStoreMessage::GetDocumentStoreState { respond_to } => {
                let _ = respond_to.send(self.document_store_state);
            }
            DocumentStoreMessage::AggressivelyCache { respond_to } => {
                let _ = respond_to.send(Ok(()));
                todo!();
            }
            DocumentStoreMessage::GetConventions { respond_to } => {
                let result = DocumentConventions;
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetDatabase {
                respond_to: _respond_to,
            } => {
                let result = None;
                let _ = _respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetDocumentStoreIdentifier { respond_to } => {
                let result = "".to_string();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetReceiverForConversionEvents { respond_to } => {
                let result = self.conversion_events_sender.subscribe();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetReceiverForCrudEvents { respond_to } => {
                let result = self.crud_events_sender.subscribe();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetReceiverForRequestEvents { respond_to } => {
                let result = self.request_events_sender.subscribe();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetReceiverForSessionEvents { respond_to } => {
                let result = self.session_events_sender.subscribe();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetUrls { respond_to } => {
                let result = Vec::<Url>::new();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::GetSubscriptions { respond_to } => {
                let result = Vec::<DocumentSubscription>::new();
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::OpenSession { respond_to } => {
                let result = DocumentSession;
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::SetConventions {
                respond_to,
                conventions,
            } => {
                let result = conventions; // TODO: return this after setting
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::SetDatabase {
                respond_to,
                database,
            } => {
                let result = database; // TODO: return this after setting
                let _ = respond_to.send(Ok(result));
                todo!();
            }
            DocumentStoreMessage::SetUrls { respond_to, urls } => {
                let result = urls; // TODO: return this after setting
                let _ = respond_to.send(Ok(result));
                todo!();
            }
        }
    }
}

async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

enum DocumentStoreMessage {
    //TODO: Consider having all of these just return the json and let the handle do
    // the data crunching and deserialization to free up the actor's message queue faster
    // -- may not be necessary with async but look into it
    AggressivelyCache {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },
    /// Requests to close its connections and destruct.
    Close {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests the [`DocumentConventions`] for this [`DocumentStore`].
    GetConventions {
        respond_to: oneshot::Sender<Result<DocumentConventions, DocumentStoreError>>,
    },

    GetDatabase {
        respond_to: oneshot::Sender<Result<Option<String>, DocumentStoreError>>,
    },

    GetDocumentStoreIdentifier {
        respond_to: oneshot::Sender<Result<String, DocumentStoreError>>,
    },

    /// Requests the [`DocumentStoreActor`]'s state.
    /// Returns: [`DocumentStoreState`]
    GetDocumentStoreState {
        respond_to: oneshot::Sender<DocumentStoreState>,
    },

    GetReceiverForConversionEvents {
        respond_to:
            oneshot::Sender<Result<broadcast::Receiver<ConversionEvents>, DocumentStoreError>>,
    },
    GetReceiverForCrudEvents {
        respond_to: oneshot::Sender<Result<broadcast::Receiver<CrudEvents>, DocumentStoreError>>,
    },
    GetReceiverForRequestEvents {
        respond_to: oneshot::Sender<Result<broadcast::Receiver<RequestEvents>, DocumentStoreError>>,
    },
    GetReceiverForSessionEvents {
        respond_to: oneshot::Sender<Result<broadcast::Receiver<SessionEvents>, DocumentStoreError>>,
    },

    /// Requests the urls of all RavenDB nodes.
    GetUrls {
        respond_to: oneshot::Sender<Result<Vec<Url>, DocumentStoreError>>,
    },

    /// Requests's [`DocumentSubscriptions`]
    GetSubscriptions {
        respond_to: oneshot::Sender<Result<Vec<DocumentSubscription>, DocumentStoreError>>,
    }, // Maybe another actor or stateful struct?

    OpenSession {
        respond_to: oneshot::Sender<Result<DocumentSession, DocumentStoreError>>,
    },

    /// Requests to set the conventions provided.
    SetConventions {
        respond_to: oneshot::Sender<Result<DocumentConventions, DocumentStoreError>>,
        conventions: DocumentConventions,
    }, // Maybe another actor or stateful struct?

    SetDatabase {
        respond_to: oneshot::Sender<Result<String, DocumentStoreError>>,
        database: String,
    },

    /// Requests to set the provided list of urls.
    SetUrls {
        respond_to: oneshot::Sender<Result<Vec<Url>, DocumentStoreError>>,
        urls: Vec<Url>,
    },
}

#[derive(Clone, Copy)]
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
    async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    cluster_urls: Vec<Url>,
    client_identity: reqwest::Identity,
}

// Placeholders below
pub struct Conventions;
pub struct CertificatePlaceholder;

pub struct DatabaseChanges;
pub struct DatabaseChangesBuilder;
pub struct DocumentConventions;
pub struct DocumentSubscription;
pub struct DocumentStoreError;

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
}
