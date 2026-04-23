use async_nats::Client;
use prost::Message;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use tonic::async_trait;
use tucana::aquila::ExecutionResult;
use tucana::shared::Value;

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
    async fn execute_remote(&self, execution: RemoteExecution) -> Result<Value, RuntimeError> {
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
                    "RemoteRuntimeExeption: failed to handle NATS message: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000001",
                    "RemoteRuntimeExeption",
                    "Failed to receive any response messages from a remote runtime.",
                ));
            }
        };

        let decode_result = ExecutionResult::decode(message.payload);
        let execution_result = match decode_result {
            Ok(r) => r,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeExeption: failed to decode NATS message: {}",
                    err
                );
                return Err(RuntimeError::new(
                    "T-PROV-000002",
                    "RemoteRuntimeExeption",
                    "Failed to read Remote Response",
                ));
            }
        };

        match execution_result.result {
            Some(result) => match result {
                tucana::aquila::execution_result::Result::Success(value) => Ok(value),
                tucana::aquila::execution_result::Result::Error(err) => {
                    let code = err.code.to_string();
                    let description = match err.description {
                        Some(string) => string,
                        None => "Unknown Error".to_string(),
                    };
                    let error = RuntimeError::new(code, "RemoteExecutionError", description);
                    Err(error)
                }
            },
            None => Err(RuntimeError::new(
                "T-PROV-000003",
                "RemoteRuntimeExeption",
                "Result of Remote Response was empty.",
            )),
        }
    }
}
