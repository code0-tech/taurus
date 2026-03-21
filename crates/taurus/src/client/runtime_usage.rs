use code0_flow::flow_service::retry::create_channel_with_retry;
use tonic::transport::Channel;
use tucana::{
    aquila::{RuntimeUsageRequest, runtime_usage_service_client::RuntimeUsageServiceClient},
    shared::RuntimeUsage,
};

pub struct TaurusRuntimeUsageService {
    channel: Channel,
}

impl TaurusRuntimeUsageService {
    pub async fn from_url(aquila_url: String) -> Self {
        let channel = create_channel_with_retry("Aquila", aquila_url).await;
        TaurusRuntimeUsageService { channel }
    }

    pub async fn update_runtime_usage(&self, runtime_usage: RuntimeUsage) {
        log::info!("Updating the current Runtime Status!");
        let mut client = RuntimeUsageServiceClient::new(self.channel.clone());

        let request = RuntimeUsageRequest {
            runtime_usage: vec![runtime_usage],
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

