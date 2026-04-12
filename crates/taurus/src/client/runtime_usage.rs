use code0_flow::flow_service::{
    auth::get_authorization_metadata, retry::create_channel_with_retry,
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{RuntimeUsageRequest, runtime_usage_service_client::RuntimeUsageServiceClient},
    shared::RuntimeUsage,
};

pub struct TaurusRuntimeUsageService {
    channel: Channel,
    aquila_token: String,
}

impl TaurusRuntimeUsageService {
    pub async fn from_url(aquila_url: String, aquila_token: String) -> Self {
        let channel = create_channel_with_retry("Aquila", aquila_url).await;
        TaurusRuntimeUsageService {
            channel,
            aquila_token,
        }
    }

    pub async fn update_runtime_usage(&self, runtime_usage: RuntimeUsage) {
        log::info!("Updating the current Runtime Status!");
        let mut client = RuntimeUsageServiceClient::new(self.channel.clone());

        let request = Request::from_parts(
            get_authorization_metadata(&self.aquila_token),
            Extensions::new(),
            RuntimeUsageRequest {
                runtime_usage: vec![runtime_usage],
            },
        );

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
