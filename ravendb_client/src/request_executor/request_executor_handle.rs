use reqwest::{Identity, Url};
use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

use crate::{document_conventions::DocumentConventions, raven_command::RavenCommand, DnsOverrides};

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
        initial_urls: Vec<Url>,
        database_name: String,
        dns_overrides: DnsOverrides,
        proxy_address: Option<String>,
        identity: Option<Identity>,
        conventions: DocumentConventions,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = RequestExecutorActor::new(
            receiver,
            database_name,
            identity,
            initial_urls.clone(),
            dns_overrides,
            proxy_address,
            conventions,
        );

        tokio::spawn(run_request_executor_actor(actor));

        // Tell the actor to do it's first topology update
        let _ =
            sender.blocking_send(RequestExecutorMessage::InitialUpdateTopology { initial_urls });

        Self { sender }
    }

    pub(crate) fn new_for_single_node_with_configuration_updates(
        url: Url,
        database_name: String,
        dns_overrides: DnsOverrides,
        proxy_address: Option<String>,
        identity: Option<Identity>,
        conventions: DocumentConventions,
    ) -> Self {
        //TODO: Finish this method
        RequestExecutor::new_for_single_node_without_configuration_updates(
            url,
            database_name,
            dns_overrides,
            proxy_address,
            identity,
            conventions,
        )
    }

    pub(crate) fn new_for_single_node_without_configuration_updates(
        url: Url,
        database_name: String,
        dns_overrides: DnsOverrides,
        proxy_address: Option<String>,
        identity: Option<Identity>,
        conventions: DocumentConventions,
    ) -> Self {
        //TODO: Finish this method
        RequestExecutor::new(
            vec![url],
            database_name,
            dns_overrides,
            proxy_address,
            identity,
            conventions,
        )
    }

    #[instrument(level = "DEBUG", skip(self))]
    pub(crate) async fn execute_request(
        &self,
        command: RavenCommand,
    ) -> Result<reqwest::Response, RequestExecutorError> {
        let (respond_to, receiver) = oneshot::channel();
        let executemsg = RequestExecutorMessage::ExecuteRavenCommand {
            respond_to,
            command,
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
