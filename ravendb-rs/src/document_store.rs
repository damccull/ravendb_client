/**
A trait representing a document store.

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
*/
pub trait DocumentStore {
    fn initialize(&self);
}

/**
[`DefaultDocumentStore`] is a provided implementation of the [`DocumentStore`].

# Example
```rust
use ravendb_client::{DocumentStore, DefaultDocumentStoreBuilder};

let docstore = DefaultDocumentStoreBuilder::new().build();
docstore.initialize();
```
*/
#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentStore {
    urls: Vec<String>,
    conventions: Option<Conventions>,
    database: Option<String>,
    certificate: Option<Certificate>,
}
impl DefaultDocumentStore {
    /// Returns a [`DefaultDocumentStoreBuilder`] to allow configuration
    /// of the DefaultDocumentStore.
    pub fn builder() -> DefaultDocumentStoreBuilder {
        DefaultDocumentStoreBuilder::default()
    }
}
impl DocumentStore for DefaultDocumentStore {
    fn initialize(&self) {
        todo!();
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentStoreBuilder {
    document_store: DefaultDocumentStore,
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

    pub fn build(&self) -> DefaultDocumentStore {
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
