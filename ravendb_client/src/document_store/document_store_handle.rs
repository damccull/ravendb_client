use anyhow::Context;
use reqwest::Url;
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

use crate::{
    raven_command::RavenCommand, run_document_store_actor, DocumentSession, DocumentStoreActor,
    DocumentStoreBuilder, DocumentStoreError, DocumentStoreInitialConfiguration,
    DocumentStoreMessage,
};

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

    #[instrument(
        level = "debug",
        name = "Actor Handle - Execute Raven Command",
        skip(self)
    )]
    pub async fn execute_raven_command(
        &self,
        raven_command: RavenCommand,
    ) -> Result<reqwest::Response, anyhow::Error> {
        tracing::trace!("Creating oneshot channel");
        let (tx, rx) = oneshot::channel();

        tracing::trace!("Sending message to actor");
        let _ = self
            .sender
            .send(DocumentStoreMessage::ExecuteRavenCommand {
                raven_command,
                respond_to: tx,
            })
            .await;

        tracing::trace!("Waiting for oneshot to return");
        rx.await?.context("DocumentStoreActor task has been killed")
    }

    #[instrument(
        level = "debug",
        name = "Actor Handle - Get Server Address",
        skip(self)
    )]
    pub async fn get_server_address(&self) -> anyhow::Result<Url> {
        tracing::debug!("Getting a server address");
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(DocumentStoreMessage::GetServerAddress { respond_to: tx })
            .await;
        rx.await?.context("DocumentStoreActor task has been killed")
    }

    pub fn open_session(&self) -> Result<DocumentSession, DocumentStoreError> {
        let session = DocumentSession::new(self.clone());
        Ok(session)
    }
}
