mod request_executor_actor;
mod request_executor_handle;

pub use request_executor_actor::RequestExecutorActor;
pub use request_executor_handle::RequestExecutor;

pub(crate) enum RequestExecutorMessage {
    ExecuteRequest(reqwest::Request),
}
