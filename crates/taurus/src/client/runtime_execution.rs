use code0_flow::flow_service::{
    auth::get_authorization_metadata, retry::create_channel_with_retry,
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{ExecutionRequest, execution_service_client::ExecutionServiceClient},
    shared::ExecutionResult,
};

pub struct TaurusRuntimeExecutionService {
    client: ExecutionServiceClient<Channel>,
    aquila_token: String,
}

impl TaurusRuntimeExecutionService {
    pub async fn from_url(aquila_url: String, aquila_token: String) -> Self {
        let channel = create_channel_with_retry("Aquila", aquila_url).await;
        let client = ExecutionServiceClient::new(channel);

        TaurusRuntimeExecutionService {
            client,
            aquila_token,
        }
    }

    pub async fn update_runtime_execution(&mut self, runtime_execution: ExecutionResult) {
        log::info!(
            "Transmitting execution result to Aquila (execution_id={}, flow_id={}, node_results={})",
            runtime_execution.execution_identifier.as_str(),
            runtime_execution.flow_id,
            runtime_execution.node_execution_results.len()
        );

        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            ExecutionRequest {
                execution_result: Some(runtime_execution),
            },
        );

        match self.client.update(request).await {
            Ok(response) => {
                log::info!(
                    "Transmitted Execution Result (success: {})",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update RuntimeExecution: {:?}", err);
            }
        }
    }
}
