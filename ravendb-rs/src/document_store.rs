use tokio::sync::{broadcast, mpsc, oneshot};

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
}
impl DocumentStoreActor {
    fn new(receiver: mpsc::Receiver<DocumentStoreMessage>) -> Self {
        Self { receiver }
    }

    fn handle_message(&mut self, msg: DocumentStoreMessage) {
        match msg {
            DocumentStoreMessage::Close => todo!(),
            DocumentStoreMessage::Initialize => todo!(),
        }
    }
}

enum DocumentStoreMessage {
    Initialize,
    Close,
}

async fn run_document_store_actor(mut actor: DocumentStoreActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}
