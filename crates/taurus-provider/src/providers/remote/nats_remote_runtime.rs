use async_nats::Client;
use prost::Message;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tonic::async_trait;
use tucana::aquila::ActionExecutionResponse;
use tucana::shared::NodeExecutionResult;

pub struct NATSRemoteRuntime {
    client: Client,
}

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
        let res = self.client.request(topic, payload.into()).await;
        let message = match res {
            Ok(r) => r,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeException: failed to handle NATS message: {}",
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
