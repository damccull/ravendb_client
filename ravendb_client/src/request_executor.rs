mod request_executor_actor;
mod request_executor_error;
mod request_executor_handle;

pub use request_executor_actor::RequestExecutorActor;
pub use request_executor_error::RequestExecutorError;
pub use request_executor_handle::RequestExecutor;
use reqwest::Url;
use tokio::sync::oneshot;

pub(crate) enum RequestExecutorMessage {
    ExecuteRequest {
        respond_to: oneshot::Sender<Result<(), RequestExecutorError>>,
        request: reqwest::Request,
    },
    FirstTopologyUpdate {
        initial_urls: Vec<Url>,
    },
}