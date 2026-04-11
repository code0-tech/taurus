use async_nats::Client;
use prost::Message;
use taurus_core::runtime::{error::RuntimeError, remote::RemoteRuntime};
use tonic::async_trait;
use tucana::{
    aquila::{ExecutionRequest, ExecutionResult},
    shared::Value,
};

pub struct RemoteNatsClient {
    client: Client,
}

impl RemoteNatsClient {
    pub fn new(client: Client) -> Self {
        RemoteNatsClient { client }
    }
}

#[async_trait]
impl RemoteRuntime for RemoteNatsClient {
    async fn execute_remote(
        &self,
        remote_name: String,
        request: ExecutionRequest,
    ) -> Result<Value, RuntimeError> {
        let topic = format!("action.{}.{}", remote_name, request.execution_identifier);
        let payload = request.encode_to_vec();
        log::info!("Publishing to topic: {}", topic);
        let res = self.client.request(topic, payload.into()).await;
        let message = match res {
            Ok(r) => r,
            Err(err) => {
                log::error!(
                    "RemoteRuntimeExeption: failed to handle NATS message: {}",
                    err
                );
                return Err(RuntimeError::simple_str(
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
                return Err(RuntimeError::simple_str(
                    "RemoteRuntimeExeption",
                    "Failed to read Remote Response",
                ));
            }
        };

        match execution_result.result {
            Some(result) => match result {
                tucana::aquila::execution_result::Result::Success(value) => Ok(value),
                tucana::aquila::execution_result::Result::Error(err) => {
                    let name = err.code.to_string();
                    let description = match err.description {
                        Some(string) => string,
                        None => "Unknown Error".to_string(),
                    };
                    let error = RuntimeError::new(name, description, None);
                    Err(error)
                }
            },
            None => Err(RuntimeError::simple_str(
                "RemoteRuntimeExeption",
                "Result of Remote Response was empty.",
            )),
        }
    }
}
