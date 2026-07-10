use std::time::{Duration, SystemTime, UNIX_EPOCH};

use code0_flow::flow_service::{
    auth::get_authorization_metadata, retry::create_channel_with_retry,
};
use tonic::{Extensions, Request, transport::Channel};
use tucana::{
    aquila::{
        RuntimeStatusUpdateRequest, runtime_status_service_client::RuntimeStatusServiceClient,
    },
    shared::{ModuleStatus, module_status::StatusVariant},
};

use crate::telemetry::errors;

pub struct TaurusRuntimeStatusService {
    channel: Channel,
    identifiers: Vec<String>,
    aquila_token: String,
}

impl TaurusRuntimeStatusService {
    pub async fn from_url(
        aquila_url: String,
        aquila_token: String,
        identifiers: Vec<String>,
        connect_timeout: Duration,
        request_timeout: Duration,
    ) -> Self {
        let channel =
            create_channel_with_retry("Aquila", aquila_url, connect_timeout, request_timeout).await;
        Self::new(channel, aquila_token, identifiers)
    }

    pub fn new(channel: Channel, aquila_token: String, identifiers: Vec<String>) -> Self {
        TaurusRuntimeStatusService {
            channel,
            identifiers,
            aquila_token,
        }
    }

    pub async fn update_runtime_status(&self, status: StatusVariant) {
        log::info!("Updating the current Runtime Status!");
        let mut client = RuntimeStatusServiceClient::new(self.channel.clone());
        let timestamp = now_unix_seconds();

        for request in build_runtime_status_requests(&self.identifiers, status, timestamp) {
            let request = Request::from_parts(
                get_authorization_metadata(&self.aquila_token),
                Extensions::new(),
                request,
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
                    errors::record(
                        "transport",
                        "aquila.runtime_status.update",
                        &err,
                        "service=runtime_status",
                    );
                }
            }
        }
    }
}

fn now_unix_seconds() -> i64 {
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_secs() as i64,
        Err(err) => {
            log::error!("cannot get current system time: {:?}", err);
            errors::record("system", "time.now", &err, "clock=system");
            0
        }
    }
}

pub(crate) fn build_runtime_status_requests(
    identifiers: &[String],
    status: StatusVariant,
    timestamp: i64,
) -> Vec<RuntimeStatusUpdateRequest> {
    identifiers
        .iter()
        .map(|identifier| RuntimeStatusUpdateRequest {
            status: Some(ModuleStatus {
                status: status.into(),
                timestamp,
                identifier: identifier.clone(),
            }),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_one_runtime_status_request_per_module_identifier() {
        let identifiers = vec!["taurus".to_string(), "http".to_string()];

        let requests = build_runtime_status_requests(&identifiers, StatusVariant::Running, 123);

        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].status,
            Some(ModuleStatus {
                identifier: "taurus".to_string(),
                timestamp: 123,
                status: StatusVariant::Running.into(),
            })
        );
        assert_eq!(
            requests[1].status,
            Some(ModuleStatus {
                identifier: "http".to_string(),
                timestamp: 123,
                status: StatusVariant::Running.into(),
            })
        );
    }
}
