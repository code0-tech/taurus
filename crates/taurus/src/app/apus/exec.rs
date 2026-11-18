//! Flow executions requested by a directly connected action (`EXEC` frames).
//! Replaces Aquila's `handle_flow_execution`: look up the flow (Aquila keeps
//! them in its flow store; Taurus caches and watches them), build the
//! `ExecutionFlow`, then run it in this process (`local`) or put it on the NATS
//! execution queue (`nats`).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use apus_protocol::ErrorInfo;
use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::engine::{ExecutionEngine, ExecutionId};
use taurus_core::types::signal::Signal;
use tucana::shared::{ExecutionFlow, ExecutionResult, ValidationFlow, Value, execution_result};

use super::remote::RoutingRemote;
use crate::app::worker;
use crate::client::runtime_usage::RuntimeUsageRecorder;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Publish on `execution.<id>` with a reply inbox; any Taurus worker runs it.
    Nats,
    /// Run the flow directly in this process (no NATS hop).
    Local,
}

pub struct ExecFailure {
    pub code: String,
    pub category: String,
    pub message: String,
}

impl ExecFailure {
    pub fn new(code: &str, category: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            category: category.to_string(),
            message: message.into(),
        }
    }

    pub fn into_error(self) -> ErrorInfo {
        ErrorInfo {
            timestamp_ms: taurus_core::time::now_unix_micros() / 1000,
            version: "taurus".to_string(),
            ..ErrorInfo::new(self.code, self.category, self.message)
        }
    }
}

/// Where Taurus gets flows from when an action asks to execute one.
pub enum FlowSource {
    /// NATS KV `flow_store` (Aquila writes it), optionally cached; the cache
    /// follows the store through a KV watch (see [`ExecContext::watch_flows`]).
    Kv {
        kv: async_nats::jetstream::kv::Store,
        cache: Option<Arc<Mutex<HashMap<i64, Arc<ValidationFlow>>>>>,
    },
    /// Held in memory (no NATS). Production: pushed by Aquila over the control plane.
    Memory(HashMap<i64, Arc<ValidationFlow>>),
}

pub struct ExecContext {
    pub client: Option<async_nats::Client>,
    pub flows: FlowSource,
    pub mode: ExecutionMode,
    pub engine: Arc<ExecutionEngine>,
    pub remote: RoutingRemote,
    pub timeout: Duration,
    pub flow_type: String,
    pub with_trace: bool,
    /// Runtime usage of local executions (dynamic mode), like the NATS worker
    /// records it; taken at shutdown so the usage publisher can flush and exit.
    pub runtime_usage: Mutex<Option<RuntimeUsageRecorder>>,
}

impl ExecContext {
    pub fn stop_runtime_usage(&self) {
        self.runtime_usage
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
    }

    pub async fn execute(
        &self,
        action: &str,
        flow_id: i64,
        project_id: i64,
        execution_id: &str,
        input: Value,
        projects: Option<&HashSet<i64>>,
    ) -> Result<Value, ExecFailure> {
        let flow = self.load_flow(flow_id).await.ok_or_else(|| {
            ExecFailure::new(
                "T-APUS-FLOW-000001",
                "NOT_FOUND",
                format!("flow {flow_id} was not found"),
            )
        })?;
        if flow.project_id != project_id {
            // The action routed by the project it claimed; it must be the flow's.
            return Err(ExecFailure::new(
                "T-APUS-FLOW-000004",
                "INVALID_ARGUMENT",
                format!("flow {flow_id} does not belong to project {project_id}"),
            ));
        }
        if flow.definition_source.as_deref() != Some(format!("action.{action}").as_str()) {
            return Err(ExecFailure::new(
                "T-APUS-FLOW-000002",
                "PERMISSION_DENIED",
                format!("flow {flow_id} does not belong to action {action}"),
            ));
        }
        if let Some(projects) = projects
            && !projects.contains(&flow.project_id)
        {
            return Err(ExecFailure::new(
                "APUS-FORBIDDEN",
                "PERMISSION_DENIED",
                format!("the ticket does not cover project {}", flow.project_id),
            ));
        }
        if let Some(reason) = &flow.disable_reason {
            return Err(ExecFailure::new(
                "T-APUS-FLOW-000003",
                "FAILED_PRECONDITION",
                format!("flow {flow_id} has been disabled: {reason}"),
            ));
        }

        let execution_flow = ExecutionFlow {
            flow_id,
            project_id: flow.project_id,
            starting_node_id: flow.starting_node_id,
            node_functions: flow.node_functions.clone(),
            input_value: Some(input),
        };

        match self.mode {
            ExecutionMode::Nats => self.execute_via_nats(execution_id, execution_flow).await,
            ExecutionMode::Local => Ok(self.execute_locally(execution_id, execution_flow).await?),
        }
    }

    async fn execute_via_nats(
        &self,
        execution_id: &str,
        flow: ExecutionFlow,
    ) -> Result<Value, ExecFailure> {
        let client = self.client.as_ref().ok_or_else(|| {
            ExecFailure::new("T-APUS-EXEC-000003", "UNAVAILABLE", "NATS is disabled")
        })?;
        let request = async_nats::Request::new()
            .payload(flow.encode_to_vec().into())
            .timeout(Some(self.timeout));
        let reply = client
            .send_request(format!("execution.{execution_id}"), request)
            .await
            .map_err(|err| {
                ExecFailure::new(
                    "T-APUS-EXEC-000001",
                    "UNAVAILABLE",
                    format!("execution request failed: {err}"),
                )
            })?;
        let result = ExecutionResult::decode(reply.payload).map_err(|err| {
            ExecFailure::new(
                "T-APUS-EXEC-000002",
                "INTERNAL",
                format!("invalid execution result: {err}"),
            )
        })?;
        match result.result {
            Some(execution_result::Result::Success(value)) => Ok(value),
            Some(execution_result::Result::Error(err)) => Err(ExecFailure {
                code: err.code,
                category: err.category,
                message: err.message,
            }),
            None => Ok(null()),
        }
    }

    async fn execute_locally(
        &self,
        execution_id: &str,
        flow: ExecutionFlow,
    ) -> Result<Value, ExecFailure> {
        let execution_id =
            ExecutionId::parse_str(execution_id).unwrap_or_else(|_| ExecutionId::new_v4());
        let function_identifiers = worker::function_identifiers_by_node_id(&flow);
        let run = worker::execute_flow(
            execution_id,
            flow,
            &self.engine,
            Some(&self.remote),
            &self.flow_type,
            function_identifiers,
            self.with_trace,
        )
        .await;
        if let Some(runtime_usage) = self
            .runtime_usage
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            runtime_usage.record(run.flow_id, run.finished_at.saturating_sub(run.started_at));
        }
        match run.signal {
            Signal::Success(value) | Signal::Return(value) => Ok(value),
            Signal::Stop => Ok(null()),
            Signal::Failure(err) => Err(ExecFailure {
                code: err.code,
                category: err.category,
                message: err.message,
            }),
        }
    }

    pub(crate) async fn load_flow(&self, flow_id: i64) -> Option<Arc<ValidationFlow>> {
        let (kv, cache) = match &self.flows {
            FlowSource::Memory(flows) => return flows.get(&flow_id).cloned(),
            FlowSource::Kv { kv, cache } => (kv, cache),
        };
        if let Some(cache) = cache
            && let Some(flow) = cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&flow_id)
        {
            return Some(flow.clone());
        }
        let bytes = kv.get(flow_id.to_string()).await.ok()??;
        let flow = Arc::new(ValidationFlow::decode(bytes).ok()?);
        if let Some(cache) = cache {
            cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(flow_id, flow.clone());
        }
        Some(flow)
    }

    /// Keeps the flow cache in sync with Aquila's flow store: updated flows are
    /// replaced, deleted ones removed (no stale flows after an edit).
    pub fn watch_flows(&self) {
        let FlowSource::Kv {
            kv,
            cache: Some(cache),
        } = &self.flows
        else {
            return;
        };
        let (kv, cache) = (kv.clone(), cache.clone());
        tokio::spawn(async move {
            let mut watch = match kv.watch_all().await {
                Ok(watch) => watch,
                Err(err) => {
                    log::warn!("Apus: cannot watch the flow store: {}", err);
                    return;
                }
            };
            while let Some(entry) = watch.next().await {
                let Ok(entry) = entry else { continue };
                let Ok(flow_id) = entry.key.parse::<i64>() else {
                    continue;
                };
                let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
                match entry.operation {
                    async_nats::jetstream::kv::Operation::Put => {
                        match ValidationFlow::decode(entry.value) {
                            Ok(flow) => {
                                cache.insert(flow_id, Arc::new(flow));
                            }
                            Err(_) => {
                                cache.remove(&flow_id);
                            }
                        }
                    }
                    _ => {
                        cache.remove(&flow_id);
                    }
                }
            }
        });
    }
}

/// Loads a JSON flow export (`tucana::shared::Flows`, same format as Aquila's static mode).
pub fn load_flows_file(path: &str) -> HashMap<i64, Arc<ValidationFlow>> {
    let data = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Apus: cannot read flows file {path}: {err}"));
    let flows: tucana::shared::Flows = serde_json::from_str(&data)
        .unwrap_or_else(|err| panic!("Apus: cannot parse flows file {path}: {err}"));
    flows
        .flows
        .into_iter()
        .map(|flow| (flow.flow_id, Arc::new(flow)))
        .collect()
}

fn null() -> Value {
    Value {
        kind: Some(tucana::shared::value::Kind::NullValue(0)),
    }
}
