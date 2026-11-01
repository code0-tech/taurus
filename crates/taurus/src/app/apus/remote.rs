//! `RoutingRemote`: sends remote node calls over Apus when the target action
//! is connected directly, and falls back to the NATS/Aquila path otherwise.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use apus_protocol::TraceContext;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::time::now_unix_micros;
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tonic::async_trait;
use tucana::shared::{NodeExecutionResult, node_execution_result};

use super::convert;
use super::hub::{ApusHub, CallReply};

#[derive(Clone)]
pub struct RoutingRemote {
    apus: Option<Arc<ApusHub>>,
    nats: Option<NATSRemoteRuntime>,
    call_timeout: Duration,
    /// Attach a W3C trace context to every `CALL` (normally: sampled executions only).
    trace: bool,
    spans: Arc<AtomicU64>,
}

impl RoutingRemote {
    pub fn new(
        apus: Option<Arc<ApusHub>>,
        nats: Option<NATSRemoteRuntime>,
        call_timeout: Duration,
        trace: bool,
    ) -> Self {
        Self {
            apus,
            nats,
            call_timeout,
            trace,
            spans: Arc::new(AtomicU64::new(1)),
        }
    }

    fn trace_context(&self, execution_id: &str) -> Option<TraceContext> {
        if !self.trace {
            return None;
        }
        let trace_id = uuid::Uuid::parse_str(execution_id)
            .map(|id| *id.as_bytes())
            .unwrap_or_default();
        Some(TraceContext {
            trace_id,
            span_id: self.spans.fetch_add(1, Ordering::Relaxed).to_le_bytes(),
            flags: 1,
        })
    }
}

#[async_trait]
impl RemoteRuntime for RoutingRemote {
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        let picked = self.apus.as_ref().and_then(|hub| {
            hub.pick(
                &execution.target_service,
                &execution.request.function_identifier,
                execution.request.project_id,
            )
        });
        let Some((connection, method)) = picked else {
            return match &self.nats {
                Some(nats) => nats.execute_remote(execution).await,
                None => Err(RuntimeError::new(
                    "T-APUS-000003",
                    "ApusNoRoute",
                    format!(
                        "action '{}' is not connected via Apus and NATS is disabled",
                        execution.target_service
                    ),
                )),
            };
        };

        let trace = self.trace_context(&execution.request.execution_identifier);
        let sent_at = now_unix_micros();
        let reply = connection
            .call(
                method,
                &execution.request,
                self.call_timeout,
                execution.sub_flow_activity.as_ref(),
                trace.as_ref(),
            )
            .await?;
        let received_at = now_unix_micros();

        // The action's own timestamps, as in `ActionExecutionResponse.node_result`.
        let (result, started_at, finished_at) = match reply {
            CallReply::Ok {
                value,
                started_at,
                finished_at,
            } => (
                node_execution_result::Result::Success(value),
                started_at,
                finished_at,
            ),
            CallReply::Err {
                error,
                started_at,
                finished_at,
            } => (
                node_execution_result::Result::Error(convert::to_tucana_error(error)),
                started_at,
                finished_at,
            ),
        };

        Ok(NodeExecutionResult {
            started_at: if started_at > 0 { started_at } else { sent_at },
            finished_at: if finished_at > 0 {
                finished_at
            } else {
                received_at
            },
            parameter_results: Vec::new(),
            id: None,
            result: Some(result),
        })
    }
}
