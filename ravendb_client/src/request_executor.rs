mod request_executor_actor;
mod request_executor_error;
mod request_executor_handle;

pub use request_executor_actor::RequestExecutorActor;
pub use request_executor_error::RequestExecutorError;
pub use request_executor_handle::RequestExecutor;
use reqwest::Url;
use tokio::sync::oneshot;

use crate::{database_topology::DatabaseTopology, raven_command::RavenCommand};

pub(crate) enum RequestExecutorMessage {
    ExecuteRavenCommand {
        respond_to: oneshot::Sender<Result<(), RequestExecutorError>>,
        request: RavenCommand,
    },
    InitialUpdateTopology {
        initial_urls: Vec<Url>,
    },
    UpdateTopology,
    TopologyUpdated {
        topology: DatabaseTopology,
    },
}
