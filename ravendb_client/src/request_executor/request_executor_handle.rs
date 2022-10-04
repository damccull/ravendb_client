use reqwest::{Identity, Url};
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

use crate::document_conventions::DocumentConventions;

use super::{
    request_executor_actor::run_request_executor_actor, RequestExecutorActor, RequestExecutorError,
    RequestExecutorMessage,
};

#[derive(Clone, Debug)]
pub struct RequestExecutor {
    sender: mpsc::Sender<RequestExecutorMessage>,
}

impl RequestExecutor {
    pub(crate) fn new(
        urls: &[Url],
        database: String,
        identity: Identity,
        conventions: DocumentConventions,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = RequestExecutorActor::new(receiver);

        tokio::spawn(run_request_executor_actor(actor));

        Self { sender }
    }

    pub(crate) fn new_for_single_node_with_configuration_updates(
        url: Url,
        database: String,
        identity: Identity,
        conventions: DocumentConventions,
    ) -> Self {
        RequestExecutor::new(&[url], database, identity, conventions)
    }

    #[instrument(level = "DEBUG", skip(self))]
    pub(crate) async fn execute_request(
        &self,
        request: reqwest::Request,
    ) -> Result<(), RequestExecutorError> {
        let (respond_to, receiver) = oneshot::channel();
        let executemsg = RequestExecutorMessage::ExecuteRequest {
            respond_to,
            request,
        };
        let _ = self.sender.send(executemsg).await;

        match receiver.await {
            Ok(r) => r,
            Err(e) => {
                Err(
                    RequestExecutorError::UnexpectedError(
                        anyhow::anyhow!(
                            "Could not receive result from request executor actor. Actor probably died. Caused by: {}",
                            e
                        )
                    )
                )
            }
        }
    }
}
