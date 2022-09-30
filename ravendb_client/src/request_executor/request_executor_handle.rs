use tokio::sync::mpsc;
use tracing::instrument;

use super::{
    request_executor_actor::run_request_executor_actor, RequestExecutorActor, RequestExecutorError,
    RequestExecutorMessage,
};

#[derive(Clone, Debug)]
pub struct RequestExecutor {
    _sender: mpsc::Sender<RequestExecutorMessage>,
}

impl RequestExecutor {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = RequestExecutorActor::new(receiver);

        tokio::spawn(run_request_executor_actor(actor));

        Self { _sender: sender }
    }

    pub(crate) fn new_for_single_node_with_configuration_updates() -> Self {
        RequestExecutor::new()
    }

    #[instrument(level = "DEBUG", skip(self))]
    pub(crate) fn execute_request(
        &self,
        request: reqwest::Request,
    ) -> Result<(), RequestExecutorError> {
        todo!()
    }
}
