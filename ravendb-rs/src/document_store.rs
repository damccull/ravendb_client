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
            DocumentStoreMessage::Close => todo!(),
            DocumentStoreMessage::Initialize => todo!(),
        }
    }
}

async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

enum DocumentStoreMessage {
    Initialize,
    Close,
}

// Placeholders below
pub struct Conventions;
pub struct CertificatePlaceholder;

pub struct DatabaseChanges;
pub struct DatabaseChangesBuilder;
