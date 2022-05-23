use crate::events::{ConversionEvents, CrudEvents, RequestEvents, SessionEvents};
use tokio::sync::broadcast;

/**
A [`DocumentStore`] is the main client API object. It establishes and
manages communication between your client application and a RavenDB
cluster.

All communication is done via HTTP requests.

The Document Store holds the Cluster Topology, the Authentication
Certificate, and any configurations & customizations that you may
have applied.

Caching is built in. All requests to the server(s) and their
responses are cached within the Document Store.

A single instance of the [`DocumentStore`] should be created per
cluster per the lifetime of your application.

WIP: It can be cloned cheaply as only a reference is given during clone.

WIP: The Document Store is thread safe - implemented in a thread safe manner.

# Event Receivers
The libraries for other languages use the observer pattern to register a callback
handler with a delegate. This doesn't fit rust well so the events have been
re-imagined using tokio broadcast channels and enums. This allows events to be
broadcast to anyone listening and they can be handled locally instead of the library
calling a function for each event.

To register for an event, call one of the `get_x_events_receiver` methods on the
[`DocumentStore`] and handle any events received through it as desired.

# Example
```rust
use ravendb_client::{DocumentStore, DefaultDocumentStoreBuilder};

let docstore = DefaultDocumentStoreBuilder::new().build();
docstore.initialize();
```
*/
#[derive(Clone, Debug)]
pub struct DocumentStore {
    crud_events_sender: broadcast::Sender<CrudEvents>,
    request_events_sender: broadcast::Sender<RequestEvents>,
    conversion_events_sender: broadcast::Sender<ConversionEvents>,
    session_events_sender: broadcast::Sender<SessionEvents>,
    urls: Vec<String>,
    conventions: Option<Conventions>,
    database: Option<String>,
    certificate: Option<Certificate>,
}
impl DocumentStore {
    /// Returns a [`DefaultDocumentStoreBuilder`] to allow configuration
    /// of the DefaultDocumentStore.
    pub fn builder() -> DefaultDocumentStoreBuilder {
        DefaultDocumentStoreBuilder::default()
    }

    pub fn initialize(&self) {
        todo!();
    }

    /// Get a [`tokio::sync::broadcast::Receiver`] of [`CrudEvents`].
    pub fn get_crud_events_receiver(&self) -> broadcast::Receiver<CrudEvents> {
        self.crud_events_sender.subscribe()
    }

    /// Get a [`tokio::sync::broadcast::Receiver`] of [`RequestEvents`]
    pub fn get_request_events_receiver(&self) -> broadcast::Receiver<RequestEvents> {
        self.request_events_sender.subscribe()
    }

    /// Get a [`tokio::sync::broadcast::Receiver`] of [`ConversionEvents`]
    pub fn get_conversion_events_receiver(&self) -> broadcast::Receiver<ConversionEvents> {
        self.conversion_events_sender.subscribe()
    }

    /// Get a [`tokio::sync::broadcast::Receiver`] of [`SessionEvents`]
    pub fn get_session_events_receiver(&self) -> broadcast::Receiver<SessionEvents> {
        self.session_events_sender.subscribe()
    }

    /// Subscribe to change notifications from the server.
    ///
    /// If `server` or `node` are [`None`], the default will be selected.
    pub fn changes(&self) -> DatabaseChangesBuilder {
        todo!()
    }
}
impl Default for DocumentStore {
    fn default() -> Self {
        let (crud_sender, _) = broadcast::channel(100);
        let (request_sender, _) = broadcast::channel(100);
        let (conversion_sender, _) = broadcast::channel(100);
        let (session_sender, _) = broadcast::channel(100);
        Self {
            crud_events_sender: crud_sender,
            request_events_sender: request_sender,
            conversion_events_sender: conversion_sender,
            session_events_sender: session_sender,
            urls: Default::default(),
            conventions: Default::default(),
            database: Default::default(),
            certificate: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentStoreBuilder {
    document_store: DocumentStore,
}
impl DefaultDocumentStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the default document store's urls.
    pub fn set_urls(&mut self, urls: Vec<String>) {
        self.document_store.urls = urls;
    }

    /// Set the default document store's conventions.
    pub fn set_conventions(&mut self, conventions: Conventions) {
        self.document_store.conventions = Some(conventions);
    }

    /// Set the default document store's database.
    pub fn set_database(&mut self, database: String) {
        self.document_store.database = Some(database);
    }

    /// Set the default document store's certificate.
    pub fn set_certificate(&mut self, certificate: Certificate) {
        self.document_store.certificate = Some(certificate);
    }

    pub fn build(&self) -> DocumentStore {
        //TODO: Change this to return a new one instead of clone
        // since clone probably needs to return a reference much
        // like Arc or similar.
        self.document_store.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Conventions;
#[derive(Clone, Debug, Default)]
pub struct Certificate;

pub struct DatabaseChanges {}
pub struct DatabaseChangesBuilder;
impl DatabaseChangesBuilder {
    pub fn server(&self, _address: &str) -> DatabaseChangesBuilder {
        todo!()
    }
    pub fn node(&self, _node_name: &str) -> DatabaseChangesBuilder {
        todo!()
    }
    pub fn build(&self) -> DatabaseChanges {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::DocumentStore;
// }
