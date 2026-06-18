use std::time::Duration;

use async_nats::Client;
use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tonic::async_trait;
use tucana::aquila::ActionExecutionResponse;
use tucana::shared::NodeExecutionResult;

pub struct NATSRemoteRuntime {
    client: Client,
}

const REMOTE_EXECUTION_RESULT_TIMEOUT: Duration = Duration::from_secs(30);

impl NATSRemoteRuntime {
    pub fn new(client: Client) -> Self {
        NATSRemoteRuntime { client }
    }
}

#[async_trait]
impl RemoteRuntime for NATSRemoteRuntime {
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        let topic = format!(
            "action.{}.{}",
            execution.target_service, execution.request.execution_identifier
        );
        let payload = execution.request.encode_to_vec();

        log::info!("Request Remote Runtime Execution with topic: : {}", topic);
        let inbox = self.client.new_inbox();
        let mut sub = match self.client.subscribe(inbox.clone()).await {
            Ok(sub) => sub,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to subscribe to NATS reply inbox: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };
        if let Err(err) = self
            .client
            .publish_with_reply(topic, inbox, payload.into())
            .await
        {
            log::error!(
                "RemoteRuntimeException: failed to publish NATS request: {}",
                err
            );
            return Err(RuntimeError::new(
                "T-PROV-000001",
                "RemoteRuntimeException",
                "Failed to receive any response messages from a remote runtime.",
            ));
        }
        match tokio::time::timeout(REMOTE_EXECUTION_RESULT_TIMEOUT, self.client.flush()).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                log::error!(
                    "RemoteRuntimeException: failed to flush NATS request: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to flush NATS request before timeout: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        }

        let message = match tokio::time::timeout(REMOTE_EXECUTION_RESULT_TIMEOUT, sub.next()).await
        {
            Ok(Some(message)) => message,
            Ok(None) => {
                log::error!("RemoteRuntimeException: NATS reply subscription closed");
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to receive NATS response before timeout: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeException",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };

        let decode_result = ActionExecutionResponse::decode(message.payload);
        match decode_result {
            Ok(r) => match r.node_result {
                Some(res) => Ok(res),
                None => {
                    log::error!("RemoteRuntimeException: received execution result without a body");
                    Err(RuntimeError::new(
                        "T-PROV-000003",
                        "RemoteRuntimeException",
                        "Received empty action execution response",
                    ))
                }
            },
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to decode NATS message: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000002",
                    "RemoteRuntimeException",
                    "Failed to read Remote Response",
                ));
            }
        }
    }
}
