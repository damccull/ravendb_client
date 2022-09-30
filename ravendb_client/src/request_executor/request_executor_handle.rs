use tokio::sync::mpsc;

use super::{
    request_executor_actor::run_request_executor_actor, RequestExecutorActor,
    RequestExecutorMessage,
};

#[derive(Clone, Debug)]
pub struct RequestExecutor {
    sender: mpsc::Sender<RequestExecutorMessage>,
}

impl RequestExecutor {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = RequestExecutorActor::new(receiver);

        tokio::spawn(run_request_executor_actor(actor));

        Self { sender }
    }
}
