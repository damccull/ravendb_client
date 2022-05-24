use tokio::sync::{broadcast, mpsc, oneshot};

use crate::events::{ConversionEvents, CrudEvents, RequestEvents, SessionEvents};

/// This a handle to the actor
#[derive(Clone)]
pub struct DocumentStore {
    sender: mpsc::Sender<DocumentStoreMessage>,
}

impl DocumentStore {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = DocumentStoreActor::new(receiver);
        tokio::spawn(run_document_store_actor(actor));

        Self { sender }
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

struct DocumentStoreActor {
    receiver: mpsc::Receiver<DocumentStoreMessage>,

    document_store_state: DocumentStoreState,

    conversion_events_sender: broadcast::Sender<ConversionEvents>,
    crud_events_sender: broadcast::Sender<CrudEvents>,
    request_events_sender: broadcast::Sender<RequestEvents>,
    session_events_sender: broadcast::Sender<SessionEvents>,

    certificate: Option<CertificatePlaceholder>,
    conventions: Option<Conventions>,
    database: Option<String>,
    trust_store: Option<CertificatePlaceholder>,
    urls: Vec<String>,
}
impl DocumentStoreActor {
    fn new(receiver: mpsc::Receiver<DocumentStoreMessage>) -> Self {
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
            urls: Default::default(),
            conventions: Default::default(),
            database: Default::default(),
            certificate: Default::default(),
            trust_store: Some(CertificatePlaceholder),
        }
    }

    fn handle_message(&mut self, msg: DocumentStoreMessage) {
        match msg {
            DocumentStoreMessage::Close { respond_to: _ } => todo!(),
            DocumentStoreMessage::Initialize { respond_to: _ } => todo!(),

            _ => todo!(),
        }
    }
}

async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

enum DocumentStoreMessage {
    AggressivelyCache {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },
    /// Requests to close its connections and destruct.
    Close {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests the [`DocumentConventions`] for this [`DocumentStore`].
    GetConventions {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    GetDatabase {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests the [`DocumentStoreActor`]'s state.
    /// Returns: [`DocumentStoreState`]
    GetDocumentStoreState {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    GetReceiverForConversionEvents {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },
    GetReceiverForCrudEvents {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },
    GetReceiverForRequestEvents {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },
    GetReveiverForSessionEvents {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests the urls of all RavenDB nodes.
    GetUrls {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests's [`DocumentSubscriptions`]
    GetSubscriptions {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    }, // Maybe another actor or stateful struct?

    /// Requests to initialize.
    Initialize {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests to set the conventions provided.
    SetConventions {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
        conventions: DocumentConventions,
    }, // Maybe another actor or stateful struct?

    SetDatabase {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
    },

    /// Requests to set the provided list of urls.
    SetUrls {
        respond_to: oneshot::Sender<Result<(), DocumentStoreError>>,
        urls: Vec<Url>,
    },
}

pub enum DocumentStoreState {
    /// [`DocumentStore`] was initialized but has since been closed.
    Closed,

    /// [`DocumentStore`] is initialized.
    Initialized,

    /// [`DocumentStore`] has not yet been initialized.
    Unitilialized,
}

// Placeholders below
pub struct Conventions;
pub struct CertificatePlaceholder;

pub struct DatabaseChanges;
pub struct DatabaseChangesBuilder;
pub struct DocumentConventions;
pub struct Url;

pub struct DocumentStoreError;
