use tokio::sync::mpsc;
use tracing::instrument;

use super::RequestExecutorMessage;

pub struct RequestExecutorActor {
    receiver: mpsc::Receiver<RequestExecutorMessage>,
}

impl RequestExecutorActor {
    pub(crate) fn new(receiver: mpsc::Receiver<RequestExecutorMessage>) -> Self {
        Self { receiver }
    }
    async fn handle_message(&self, msg: RequestExecutorMessage) {
        match msg {
            RequestExecutorMessage::_ExecuteRequest(_request) => todo!(),
        }
    }
}

#[instrument(level = "debug", name = "Running Document Store Actor", skip(actor))]
pub async fn run_request_executor_actor(mut actor: RequestExecutorActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}
