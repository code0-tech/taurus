use std::time::{SystemTime, UNIX_EPOCH};

use tonic::transport::Channel;
use tucana::{
    aquila::{
        RuntimeStatusUpdateRequest, runtime_status_service_client::RuntimeStatusServiceClient,
        runtime_status_update_request::Status,
    },
    shared::{ExecutionRuntimeStatus, RuntimeFeature},
};

pub struct TaurusRuntimeStatusService {
    channel: Channel,
    identifier: String,
    features: Vec<RuntimeFeature>,
}

impl TaurusRuntimeStatusService {
    pub async fn from_url(
        aquila_url: String,
        identifier: String,
        features: Vec<RuntimeFeature>,
    ) -> Self {
        let channel = create_channel_with_retry("Aquila", aquila_url).await;
        Self::new(channel, identifier, features)
    }

    pub fn new(
        channel: Channel,
        identifier: String,
        features: Vec<RuntimeFeature>,
    ) -> Self {
        TaurusRuntimeStatusService {
            channel,
            identifier,
            features,
        }
    }

    pub async fn update_runtime_status(
        &self,
        status: tucana::shared::execution_runtime_status::Status,
    ) {
        log::info!("Updating the current Runtime Status!");
        let mut client = RuntimeStatusServiceClient::new(self.channel.clone());

        let now = SystemTime::now();
        let timestamp = match now.duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_secs(),
            Err(err) => {
                log::error!("cannot get current system time: {:?}", err);
                0
            }
        };

        let request = RuntimeStatusUpdateRequest {
            status: Some(Status::ExecutionRuntimeStatus(ExecutionRuntimeStatus {
                status: status.into(),
                timestamp: timestamp as i64,
                identifier: self.identifier.clone(),
                features: self.features.clone(),
            })),
        };

        match client.update(request).await {
            Ok(response) => {
                log::info!(
                    "Was the update of the RuntimeStatus accepted by Sagittarius? {}",
                    response.into_inner().success
                );
            }
            Err(err) => {
                log::error!("Failed to update RuntimeStatus: {:?}", err);
            }
        }
    }
}
