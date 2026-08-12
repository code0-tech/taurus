//! The NATS-driven flow execution loop: subscribes to `execution.*`, decodes
//! each message into an `ExecutionFlow`, runs it through
//! `taurus_core::runtime::engine::ExecutionEngine`, and (in dynamic mode)
//! reports the result back to Aquila via [`TaurusRuntimeExecutionService`].
//!
//! Also subscribes to `sub_flow_execution.*`: Aquila forwards
//! `ActionSubFlowExecutionRequest`s there (NATS request/reply, not the
//! publish-then-reply-subject shape `execution.*` uses) whenever an action
//! invokes a sub-flow reference minted by a remote node call (see
//! `taurus_core::runtime::engine::sub_flow_registry`). The same
//! `execution_identifier` may be invoked any number of times while the
//! parent remote call is outstanding, so this subscriber never removes a
//! registry entry itself -- only the parent call's own resolution does
//! (`EngineExecutor::execute_remote_node`).
//!
//! Flows execute concurrently: [`spawn_worker`]'s loop only decodes and
//! dispatches messages, spawning one task per execution rather than
//! awaiting each one inline, so a slow flow doesn't stall unrelated ones.
//! Concurrency is bounded by a semaphore (`Config::max_concurrent_executions`)
//! so a burst of NATS messages can't spawn unbounded in-flight executions.
//! Once a message is dequeued from NATS we always run it to completion
//! (waiting for a permit if needed) rather than dropping it on shutdown,
//! since core NATS has no redelivery for an already-claimed message.
//! `sub_flow_execution.*` messages are expected to be quick (Phase 0's
//! aquila-side dispatch timeout keeps each hop short) so they are spawned
//! without going through the same semaphore as full flow executions.
//!
//! Callers signal shutdown cooperatively (see
//! [`crate::app::wait_for_shutdown`]) so the worker stops accepting new
//! messages and waits for every in-flight execution to finish.

use futures_lite::StreamExt;
use prost::Message;
use std::sync::Arc;
use taurus_core::runtime::engine::{ExecutionEngine, ExecutionId};
use taurus_core::runtime::remote::RemoteRuntime;
use taurus_core::time::now_unix_micros;
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::sync::{Notify, Semaphore};
use tokio::task::{JoinHandle, JoinSet};
use tucana::aquila::ActionSubFlowExecutionRequest;
use tucana::shared::execution_result;
use tucana::shared::{ExecutionFlow, ExecutionResult, NodeExecutionResult, Value};

use crate::client::runtime_execution::TaurusRuntimeExecutionService;
use crate::telemetry::{errors, metrics};

pub fn spawn_worker(
    client: async_nats::Client,
    engine: ExecutionEngine,
    nats_remote: NATSRemoteRuntime,
    runtime_execution_service: Option<TaurusRuntimeExecutionService>,
    flow_type: String,
    shutdown: Arc<Notify>,
    max_concurrent_executions: usize,
    with_trace: bool,
) -> JoinHandle<()> {
    let engine = Arc::new(engine);
    let semaphore = Arc::new(Semaphore::new(max_concurrent_executions.max(1)));

    tokio::spawn(async move {
        let mut execution_subscription = match client
            .queue_subscribe(String::from("execution.*"), "taurus".into())
            .await
        {
            Ok(subscription) => {
                log::info!("Subscribed to 'execution.*'");
                subscription
            }
            Err(err) => {
                log::error!("Failed to subscribe to 'execution.*': {:?}", err);
                errors::record(
                    "transport",
                    "nats.subscribe",
                    &err,
                    "subject=execution.* queue=taurus",
                );
                return;
            }
        };

        let mut sub_flow_execution_subscription = match client
            .queue_subscribe(String::from("sub_flow_execution.*"), "taurus".into())
            .await
        {
            Ok(subscription) => {
                log::info!("Subscribed to 'sub_flow_execution.*'");
                subscription
            }
            Err(err) => {
                log::error!("Failed to subscribe to 'sub_flow_execution.*': {:?}", err);
                errors::record(
                    "transport",
                    "nats.subscribe",
                    &err,
                    "subject=sub_flow_execution.* queue=taurus",
                );
                return;
            }
        };

        let mut execution_closed = false;
        let mut sub_flow_execution_closed = false;
        let mut in_flight = JoinSet::new();

        while !execution_closed || !sub_flow_execution_closed {
            tokio::select! {
                message = execution_subscription.next(), if !execution_closed => {
                    match message {
                        Some(message) => {
                            let Ok(permit) = semaphore.clone().acquire_owned().await else {
                                // Semaphore is never explicitly closed; unreachable in practice.
                                continue;
                            };
                            let engine = engine.clone();
                            let nats_remote = nats_remote.clone();
                            let runtime_execution_service = runtime_execution_service.clone();
                            let flow_type = flow_type.clone();
                            in_flight.spawn(async move {
                                let _permit = permit;
                                process_execution_message(
                                    message,
                                    &engine,
                                    &nats_remote,
                                    runtime_execution_service,
                                    flow_type.as_str(),
                                    with_trace,
                                ).await;
                            });
                        }
                        None => {
                            execution_closed = true;
                            log::warn!("Subscription 'execution.*' ended");
                        }
                    }
                }
                message = sub_flow_execution_subscription.next(), if !sub_flow_execution_closed => {
                    match message {
                        Some(message) => {
                            // Deliberately not gated by `semaphore`: sub-flow
                            // hops are meant to be quick (see module docs),
                            // and gating them behind the same permit pool as
                            // full flow executions could deadlock a flow
                            // that's waiting on its own sub-flow traffic
                            // while holding a permit for the parent call.
                            let engine = engine.clone();
                            let nats_remote = nats_remote.clone();
                            let client = client.clone();
                            in_flight.spawn(async move {
                                process_sub_flow_execution_message(
                                    message,
                                    &engine,
                                    &nats_remote,
                                    &client,
                                    with_trace,
                                ).await;
                            });
                        }
                        None => {
                            sub_flow_execution_closed = true;
                            log::warn!("Subscription 'sub_flow_execution.*' ended");
                        }
                    }
                }
                _ = shutdown.notified() => {
                    execution_closed = true;
                    sub_flow_execution_closed = true;
                    log::info!("NATS worker received shutdown signal");
                }
            }
        }

        log::info!(
            "NATS worker loop ended, draining {} in-flight execution(s)",
            in_flight.len()
        );
        while in_flight.join_next().await.is_some() {}
    })
}

async fn process_execution_message(
    message: async_nats::Message,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    mut runtime_execution_service: Option<TaurusRuntimeExecutionService>,
    flow_type: &str,
    with_trace: bool,
) {
    let requested_execution_id = parse_execution_id_from_subject(&message.subject, "execution")
        .unwrap_or_else(|| {
            let generated = ExecutionId::new_v4();
            log::warn!(
                "Expected subject format 'execution.<uuid>', got '{}'; generated execution id {}",
                message.subject,
                generated
            );
            generated
        });

    let flow: ExecutionFlow = match ExecutionFlow::decode(&*message.payload) {
        Ok(flow) => flow,
        Err(err) => {
            log::error!(
                "Failed to deserialize flow: {:?}, payload: {:?}",
                err,
                &message.payload
            );
            errors::record(
                "serialization",
                "execution_flow.decode",
                &err,
                format!(
                    "subject={} payload_bytes={}",
                    message.subject,
                    message.payload.len()
                ),
            );
            if let Some(execution_service) = runtime_execution_service.as_mut() {
                execution_service
                    .update_runtime_execution(build_decode_error_result(requested_execution_id))
                    .await;
            }
            return;
        }
    };

    let flow_id = flow.flow_id;
    let function_identifiers = function_identifiers_by_node_id(&flow);
    let run_result = execute_flow(
        requested_execution_id,
        flow,
        engine,
        Some(nats_remote),
        flow_type,
        function_identifiers,
        with_trace,
    )
    .await;
    log::debug!(
        "Flow {} execution completed; no direct reply message published",
        flow_id
    );

    let execution_result = build_execution_result(
        run_result.execution_id,
        run_result.flow_id,
        run_result.started_at,
        run_result.finished_at,
        run_result.input.clone(),
        run_result.node_execution_results,
        run_result.signal.clone(),
    );

    if let Some(execution_service) = runtime_execution_service.as_mut() {
        execution_service
            .update_runtime_execution(execution_result)
            .await;
    }
}

/// Handles one `sub_flow_execution.*` NATS request/reply message: decodes it
/// into an `ActionSubFlowExecutionRequest`, runs the referenced sub-flow
/// node range (if the registry still has an entry for it), and publishes an
/// `ExecutionResult` back to the request's reply subject -- aquila's
/// `nats_bridge::handle_sub_flow_execution` awaits exactly this shape via
/// `nats_client.request(...)`.
async fn process_sub_flow_execution_message(
    message: async_nats::Message,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    client: &async_nats::Client,
    with_trace: bool,
) {
    let Some(reply) = message.reply.clone() else {
        log::error!(
            "Received 'sub_flow_execution' message without a reply subject on '{}'; dropping",
            message.subject
        );
        return;
    };

    let request = match ActionSubFlowExecutionRequest::decode(&*message.payload) {
        Ok(request) => request,
        Err(err) => {
            log::error!(
                "Failed to deserialize sub flow execution request: {:?}, payload: {:?}",
                err,
                &message.payload
            );
            errors::record(
                "serialization",
                "action_sub_flow_execution_request.decode",
                &err,
                format!(
                    "subject={} payload_bytes={}",
                    message.subject,
                    message.payload.len()
                ),
            );
            publish_sub_flow_execution_result(client, reply, build_sub_flow_decode_error_result())
                .await;
            return;
        }
    };

    let result =
        build_sub_flow_execution_result(request, engine, Some(nats_remote), with_trace).await;
    publish_sub_flow_execution_result(client, reply, result).await;
}

/// Core logic for one sub-flow execution reply, decoupled from NATS wiring
/// (and, via `remote` being the `RemoteRuntime` trait object rather than the
/// concrete `NATSRemoteRuntime`, from a real NATS connection too) so it's
/// directly unit-testable: looks the id up in the engine's sub-flow
/// registry and runs it if found. A lookup miss (already completed, never
/// minted, or minted by a since-restarted process) is reported as a normal
/// error `ExecutionResult`, not a dropped/timed-out request -- so aquila
/// surfaces a clean failure to the action instead of its own request timing
/// out silently (see `nats_bridge::handle_sub_flow_execution`).
async fn build_sub_flow_execution_result(
    request: ActionSubFlowExecutionRequest,
    engine: &ExecutionEngine,
    remote: Option<&dyn RemoteRuntime>,
    with_trace: bool,
) -> ExecutionResult {
    let execution_identifier = request.execution_identifier.clone();
    let started_at = now_unix_micros();

    match engine
        .execute_sub_flow(&execution_identifier, request.parameters, remote, with_trace)
        .await
    {
        Some(report) => build_execution_result(
            execution_id_or_generate(&execution_identifier),
            0,
            started_at,
            now_unix_micros(),
            None,
            report.node_execution_results,
            report.signal,
        ),
        None => build_sub_flow_not_found_result(execution_identifier, started_at),
    }
}

/// The registry mints ids via `uuid::Uuid::new_v4()`, so parsing back should
/// always succeed; falls back to a fresh id only defensively (the parse
/// failure would already be visible in the id echoed back on the result).
fn execution_id_or_generate(execution_identifier: &str) -> ExecutionId {
    ExecutionId::parse_str(execution_identifier).unwrap_or_else(|_| ExecutionId::new_v4())
}

async fn publish_sub_flow_execution_result(
    client: &async_nats::Client,
    reply: async_nats::Subject,
    result: ExecutionResult,
) {
    if let Err(err) = client.publish(reply, result.encode_to_vec().into()).await {
        log::error!("Failed to publish sub flow execution result: {:?}", err);
        errors::record(
            "transport",
            "nats.publish",
            &err,
            "subject=sub_flow_execution.reply",
        );
        return;
    }
    if let Err(err) = client.flush().await {
        log::error!("Failed to flush sub flow execution result: {:?}", err);
        errors::record(
            "transport",
            "nats.flush",
            &err,
            "subject=sub_flow_execution.reply",
        );
    }
}

fn build_sub_flow_not_found_result(execution_identifier: String, started_at: i64) -> ExecutionResult {
    let now = now_unix_micros();
    let runtime_error = RuntimeError::new(
        "T-TAURUS-000002",
        "SubFlowExecutionNotFound",
        "No pending sub flow execution found for this execution identifier -- it may have \
         already completed, never existed, or the owning process restarted",
    );

    ExecutionResult {
        execution_identifier,
        flow_id: 0,
        started_at,
        finished_at: now,
        input: None,
        node_execution_results: Vec::new(),
        result: Some(execution_result::Result::Error(
            runtime_error.as_tucana_error(),
        )),
    }
}

fn build_sub_flow_decode_error_result() -> ExecutionResult {
    let now = now_unix_micros();
    let runtime_error = RuntimeError::new(
        "T-TAURUS-000003",
        "SubFlowExecutionRequestDecodeError",
        "Failed to decode sub flow execution request payload",
    );

    ExecutionResult {
        execution_identifier: String::new(),
        flow_id: 0,
        started_at: now,
        finished_at: now,
        input: None,
        node_execution_results: Vec::new(),
        result: Some(execution_result::Result::Error(
            runtime_error.as_tucana_error(),
        )),
    }
}

#[derive(Clone)]
struct FlowRunResult {
    execution_id: ExecutionId,
    flow_id: i64,
    started_at: i64,
    finished_at: i64,
    input: Option<Value>,
    signal: Signal,
    node_execution_results: Vec<NodeExecutionResult>,
}

async fn execute_flow(
    execution_id: ExecutionId,
    flow: ExecutionFlow,
    engine: &ExecutionEngine,
    remote: Option<&dyn RemoteRuntime>,
    flow_type: &str,
    function_identifiers: std::collections::HashMap<i64, String>,
    with_trace: bool,
) -> FlowRunResult {
    let started_at = now_unix_micros();
    let flow_id = flow.flow_id;
    let project_id = flow.project_id;
    let input = flow.input_value.clone();
    // Trace V2 collection is O(n^2) in executed nodes (see taurus-core's
    // ValueStore::trace_snapshot); only worth paying for in development,
    // where engine.rs prints the trace via `with_trace`.
    let report = engine
        .execute_flow_report_async(&execution_id.to_string(), flow, remote, with_trace)
        .await;
    let finished_at = now_unix_micros();
    record_flow_metrics(
        flow_id,
        project_id,
        flow_type,
        started_at,
        finished_at,
        &report.signal,
        &report.node_execution_results,
        &function_identifiers,
    );

    FlowRunResult {
        execution_id,
        flow_id,
        started_at,
        finished_at,
        input,
        signal: report.signal,
        node_execution_results: report.node_execution_results,
    }
}

fn function_identifiers_by_node_id(flow: &ExecutionFlow) -> std::collections::HashMap<i64, String> {
    flow.node_functions
        .iter()
        .filter_map(|function| {
            function
                .database_id
                .map(|id| (id, function.runtime_function_id.clone()))
        })
        .collect()
}

fn record_flow_metrics(
    flow_id: i64,
    project_id: i64,
    flow_type: &str,
    started_at: i64,
    finished_at: i64,
    signal: &Signal,
    node_execution_results: &[NodeExecutionResult],
    function_identifiers: &std::collections::HashMap<i64, String>,
) {
    metrics::flow_execution(metrics::FlowExecution {
        flow_id,
        project_id,
        flow_type,
        outcome: signal_outcome(signal),
        duration_seconds: metrics::duration_seconds(started_at, finished_at),
    });

    for result in node_execution_results {
        let node_id = metrics::result_node_id(result);
        let function_identifier = metrics::result_function_identifier(result)
            .map(ToOwned::to_owned)
            .or_else(|| node_id.and_then(|id| function_identifiers.get(&id).cloned()))
            .unwrap_or_else(|| "unknown".to_string());
        let (error_code, error_category) = metrics::node_result_error(result);

        metrics::function_execution(metrics::FunctionExecution {
            flow_id,
            project_id,
            flow_type,
            function_identifier: function_identifier.as_str(),
            node_id,
            outcome: metrics::node_result_outcome(result),
            duration_seconds: metrics::duration_seconds(result.started_at, result.finished_at),
            error_code,
            error_category,
        });
    }
}

fn signal_outcome(signal: &Signal) -> &'static str {
    match signal {
        Signal::Success(_) => "success",
        Signal::Failure(_) => "failure",
        Signal::Return(_) => "return",
        Signal::Stop => "stop",
    }
}

fn parse_execution_id_from_subject(
    subject: &async_nats::Subject,
    prefix: &str,
) -> Option<ExecutionId> {
    let raw = subject.as_str();
    let mut parts = raw.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(found_prefix), Some(uuid), None) if found_prefix == prefix => {
            ExecutionId::parse_str(uuid).ok()
        }
        _ => None,
    }
}

fn build_execution_result(
    execution_id: ExecutionId,
    flow_id: i64,
    started_at: i64,
    finished_at: i64,
    input: Option<Value>,
    node_execution_results: Vec<NodeExecutionResult>,
    signal: Signal,
) -> ExecutionResult {
    let result = match signal {
        Signal::Success(value) | Signal::Return(value) => {
            Some(execution_result::Result::Success(value))
        }
        Signal::Failure(err) => Some(execution_result::Result::Error(err.as_tucana_error())),
        Signal::Stop => Some(execution_result::Result::Success(Value {
            kind: Some(tucana::shared::value::Kind::NullValue(0)),
        })),
    };

    ExecutionResult {
        execution_identifier: execution_id.to_string(),
        flow_id,
        started_at,
        finished_at,
        input,
        node_execution_results,
        result,
    }
}

fn build_decode_error_result(execution_id: ExecutionId) -> ExecutionResult {
    let now = now_unix_micros();
    let runtime_error = RuntimeError::new(
        "T-TAURUS-000001",
        "ExecutionFlowDecodeError",
        "Failed to decode execution flow payload",
    );

    ExecutionResult {
        execution_identifier: execution_id.to_string(),
        flow_id: 0,
        started_at: now,
        finished_at: now,
        input: None,
        node_execution_results: Vec::new(),
        result: Some(execution_result::Result::Error(
            runtime_error.as_tucana_error(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tonic::async_trait;
    use serde::Deserialize;
    use std::sync::Mutex as StdMutex;
    use taurus_core::runtime::engine::ExecutionEngine;
    use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
    use taurus_core::types::exit_reason::ExitReason;
    use tucana::aquila::{ActionNodeSubFlowValue, action_node_value};
    use tucana::shared::{
        NodeFunction, NodeParameter, ValidationFlow, execution_result,
        helper::value::{from_json_value, to_json_value},
        node_value, reference_value,
        sub_flow::ExecutionReference,
        value::Kind,
    };

    #[derive(Deserialize)]
    struct FixtureInput {
        input: Option<serde_json::Value>,
        expected_result: serde_json::Value,
    }

    #[derive(Deserialize)]
    struct FlowFixture {
        inputs: Vec<FixtureInput>,
        flow: ValidationFlow,
    }

    #[test]
    fn build_execution_result_preserves_success_payload() {
        let execution_id = ExecutionId::new_v4();
        let fixture = load_fixture("flows/0003_for_each.json");
        let expected_result = fixture.inputs[0].expected_result.clone();
        let flow = execution_flow_from_fixture(fixture);
        let success = from_json_value(expected_result.clone());

        let response = build_execution_result(
            execution_id,
            flow.flow_id,
            1,
            2,
            flow.input_value,
            Vec::new(),
            Signal::Success(success),
        );

        assert_eq!(response.execution_identifier, execution_id.to_string());
        assert_eq!(response.flow_id, flow.flow_id);
        assert_eq!(response.started_at, 1);
        assert_eq!(response.finished_at, 2);
        assert!(response.node_execution_results.is_empty());

        match response.result {
            Some(execution_result::Result::Success(value)) => {
                assert_eq!(to_json_value(value), expected_result);
            }
            other => panic!("expected successful execution result, got {:?}", other),
        }
    }

    #[test]
    fn build_decode_error_result_uses_execution_payload_message() {
        let execution_id = ExecutionId::new_v4();
        let response = build_decode_error_result(execution_id);

        assert_eq!(response.execution_identifier, execution_id.to_string());
        assert_eq!(response.flow_id, 0);
        assert!(response.started_at > 0);
        assert!(response.finished_at >= response.started_at);
        assert_eq!(response.input, None);
        assert!(response.node_execution_results.is_empty());

        match response.result {
            Some(execution_result::Result::Error(error)) => {
                assert_eq!(error.code, "T-TAURUS-000001");
                assert_eq!(error.category, "ExecutionFlowDecodeError");
                assert_eq!(error.message, "Failed to decode execution flow payload");
            }
            other => panic!("expected decode error result, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn execute_flow_reports_microsecond_timestamps_and_duration() {
        let execution_id = ExecutionId::new_v4();
        let fixture = load_fixture("flows/0003_for_each.json");
        let flow = execution_flow_from_fixture(fixture);
        let engine = ExecutionEngine::new();

        let function_identifiers = function_identifiers_by_node_id(&flow);
        let run_result = execute_flow(
            execution_id,
            flow,
            &engine,
            None,
            "test",
            function_identifiers,
            false,
        )
        .await;

        println!(
            "started_at={} finished_at={} delta={}",
            run_result.started_at,
            run_result.finished_at,
            run_result.finished_at - run_result.started_at,
        );

        assert_eq!(run_result.execution_id, execution_id);
        assert!(run_result.started_at >= 1_000_000_000_000_000);
        assert!(run_result.finished_at >= run_result.started_at);
        assert!(
            run_result
                .node_execution_results
                .iter()
                .all(|result| result.started_at >= 1_000_000_000_000_000
                    && result.finished_at >= result.started_at)
        );
    }

    #[test]
    fn build_execution_result_preserves_node_execution_results() {
        let execution_id = ExecutionId::new_v4();
        let node_result = NodeExecutionResult {
            started_at: 1,
            finished_at: 2,
            parameter_results: vec![tucana::shared::NodeParameterNodeExecutionResult {
                value: Some(from_json_value(serde_json::json!("parameter-value"))),
            }],
            id: Some(tucana::shared::node_execution_result::Id::NodeId(42)),
            result: Some(tucana::shared::node_execution_result::Result::Success(
                from_json_value(serde_json::json!("node-output")),
            )),
        };

        let response = build_execution_result(
            execution_id,
            10,
            1,
            2,
            None,
            vec![node_result.clone()],
            Signal::Success(from_json_value(serde_json::json!("flow-output"))),
        );

        assert_eq!(response.node_execution_results, vec![node_result]);
    }

    fn load_fixture(path: &str) -> FlowFixture {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
        serde_json::from_str(&content)
            .unwrap_or_else(|err| panic!("failed to parse fixture {}: {err}", path.display()))
    }

    fn execution_flow_from_fixture(fixture: FlowFixture) -> ExecutionFlow {
        ExecutionFlow {
            flow_id: fixture.flow.flow_id,
            project_id: fixture.flow.project_id,
            starting_node_id: fixture.flow.starting_node_id,
            node_functions: fixture.flow.node_functions,
            input_value: fixture
                .inputs
                .first()
                .and_then(|input| input.input.clone().map(from_json_value)),
        }
    }

    // --- sub_flow_execution.* subscriber core logic -----------------------

    fn int_value(value: i64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(tucana::shared::NumberValue {
                number: Some(tucana::shared::number_value::Number::Integer(value)),
            })),
        }
    }

    fn node(
        database_id: i64,
        runtime_function_id: &str,
        parameters: Vec<NodeParameter>,
        next_node_id: Option<i64>,
        definition_source: Option<&str>,
    ) -> NodeFunction {
        NodeFunction {
            database_id: Some(database_id),
            runtime_function_id: runtime_function_id.to_string(),
            parameters,
            next_node_id,
            definition_source: definition_source.map(str::to_string),
        }
    }

    /// A `SubFlow{starting_node_id}`-valued parameter, exactly what
    /// `resolve_remote_args` mints a UUID for when the owning node is
    /// dispatched remotely (see `taurus-core::runtime::engine::executor`).
    fn sub_flow_param(database_id: i64, runtime_parameter_id: &str, node_id: i64) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(tucana::shared::NodeValue {
                value: Some(node_value::Value::SubFlow(tucana::shared::SubFlow {
                    input_schema: None,
                    output_schema: None,
                    signature: String::new(),
                    settings: Vec::new(),
                    execution_reference: Some(ExecutionReference::StartingNodeId(node_id)),
                })),
            }),
            cast: None,
        }
    }

    /// Reads the first positional value the sub-flow was invoked with --
    /// `ExecutionEngine::execute_sub_flow` seeds those as
    /// `InputType{node_id: caller_node_id, parameter_index:
    /// caller_parameter_index, input_index}`, the same addressing a local
    /// consumer callback gets (see `functions/array.rs::run_with_unary_input`),
    /// keyed here against the node/parameter that minted the sub-flow
    /// reference in the first place (node `caller_node_id`'s parameter at
    /// `caller_parameter_index`).
    fn input_type_param(
        database_id: i64,
        runtime_parameter_id: &str,
        caller_node_id: i64,
        caller_parameter_index: i64,
        input_index: i64,
    ) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(tucana::shared::NodeValue {
                value: Some(node_value::Value::ReferenceValue(
                    tucana::shared::ReferenceValue {
                        target: Some(reference_value::Target::InputType(
                            tucana::shared::InputType {
                                node_id: caller_node_id,
                                parameter_index: caller_parameter_index,
                                input_index,
                            },
                        )),
                        paths: vec![],
                    },
                )),
            }),
            cast: None,
        }
    }

    fn first_input_type_param(
        database_id: i64,
        runtime_parameter_id: &str,
        caller_node_id: i64,
        caller_parameter_index: i64,
    ) -> NodeParameter {
        input_type_param(
            database_id,
            runtime_parameter_id,
            caller_node_id,
            caller_parameter_index,
            0,
        )
    }

    /// Captures the minted sub-flow UUID from the outgoing remote request's
    /// `SubFlow` parameter, then blocks until `release` is notified --
    /// standing in for the parent remote call staying outstanding while the
    /// action drives `sub_flow_execution.*` traffic against the minted id.
    /// The registry only drops an entry once the *parent* call resolves
    /// (`EngineExecutor::execute_remote_node`), so without this the entry
    /// would already be gone by the time the test tries a second lookup.
    struct BlockingMintCapturingRuntime {
        result: NodeExecutionResult,
        minted_id: Arc<StdMutex<Option<String>>>,
        release: Arc<Notify>,
    }

    #[async_trait]
    impl RemoteRuntime for BlockingMintCapturingRuntime {
        async fn execute_remote(
            &self,
            execution: RemoteExecution,
        ) -> Result<NodeExecutionResult, RuntimeError> {
            if let Some(parameter) = execution.request.parameters.first()
                && let Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                    execution_identifier,
                })) = &parameter.value
            {
                *self
                    .minted_id
                    .lock()
                    .expect("mint recorder should not be poisoned") = Some(execution_identifier.clone());
            }
            self.release.notified().await;
            Ok(self.result.clone())
        }
    }

    async fn wait_for_minted_id(minted_id: &StdMutex<Option<String>>) -> String {
        for _ in 0..1000 {
            if let Some(id) = minted_id
                .lock()
                .expect("mint recorder should not be poisoned")
                .clone()
            {
                return id;
            }
            tokio::task::yield_now().await;
        }
        panic!("timed out waiting for a sub flow id to be minted");
    }

    #[tokio::test]
    async fn sub_flow_execution_result_runs_pending_entry_and_can_be_invoked_repeatedly() {
        let engine = Arc::new(ExecutionEngine::new());
        let minted_id = Arc::new(StdMutex::new(None));
        let release = Arc::new(Notify::new());
        let remote = BlockingMintCapturingRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(tucana::shared::node_execution_result::Id::NodeId(1)),
                result: Some(tucana::shared::node_execution_result::Result::Success(
                    int_value(1),
                )),
            },
            minted_id: Arc::clone(&minted_id),
            release: Arc::clone(&release),
        };

        let remote_node = node(
            1,
            "remote::open_stream",
            vec![sub_flow_param(100, "on_message", 2)],
            None,
            Some("action.svc"),
        );
        // Echoes the first action-supplied sub-flow-invocation parameter
        // straight back out, so a run's result proves which parameters it
        // was actually seeded with.
        let sub_flow_target = node(
            2,
            "std::control::value",
            vec![first_input_type_param(200, "value", 1, 0)],
            None,
            None,
        );

        let flow = ExecutionFlow {
            flow_id: 1,
            project_id: 7,
            starting_node_id: 1,
            node_functions: vec![remote_node, sub_flow_target],
            input_value: None,
        };

        // The "parent" remote call is kept outstanding (parked on
        // `release.notified()`) for the rest of this test by running it on
        // its own task.
        let parent_engine = Arc::clone(&engine);
        let parent = tokio::spawn(async move {
            parent_engine
                .execute_flow_report_async("parent-1", flow, Some(&remote), false)
                .await
        });

        let execution_identifier = wait_for_minted_id(&minted_id).await;

        let first_request = ActionSubFlowExecutionRequest {
            execution_identifier: execution_identifier.clone(),
            parameters: vec![int_value(41)],
        };
        let first_result =
            build_sub_flow_execution_result(first_request, &engine, None, false).await;
        assert_eq!(first_result.execution_identifier, execution_identifier);
        match first_result.result {
            Some(execution_result::Result::Success(value)) => assert_eq!(value, int_value(41)),
            other => panic!("expected success result, got {:?}", other),
        }

        // The same id can be invoked again while the parent call is still
        // outstanding -- the registry entry is not removed by running it.
        let second_request = ActionSubFlowExecutionRequest {
            execution_identifier: execution_identifier.clone(),
            parameters: vec![int_value(99)],
        };
        let second_result =
            build_sub_flow_execution_result(second_request, &engine, None, false).await;
        match second_result.result {
            Some(execution_result::Result::Success(value)) => assert_eq!(value, int_value(99)),
            other => panic!("expected success result on repeated invocation, got {:?}", other),
        }

        // Letting the parent call resolve removes the entry it minted.
        release.notify_one();
        let report = parent.await.expect("parent task should not panic");
        assert_eq!(report.exit_reason, ExitReason::Success);

        let after_parent_completion = ActionSubFlowExecutionRequest {
            execution_identifier: execution_identifier.clone(),
            parameters: Vec::new(),
        };
        let after_result =
            build_sub_flow_execution_result(after_parent_completion, &engine, None, false).await;
        match after_result.result {
            Some(execution_result::Result::Error(err)) => {
                assert_eq!(err.code, "T-TAURUS-000002");
            }
            other => panic!(
                "expected the registry entry to be gone once the parent call resolved, got {:?}",
                other
            ),
        }
    }

    /// A sub-flow's action-supplied parameters are seeded one `InputType`
    /// slot per positional value (`input_index` 0, 1, ...), not just the
    /// first. Drives a two-argument call and reads both slots back through
    /// `std::number::add`, so a regression that only seeds `input_index: 0`
    /// (as a naive port of the single-argument `for_each` case might) would
    /// fail this by treating the second argument as missing.
    #[tokio::test]
    async fn sub_flow_execution_seeds_every_positional_input_index_independently() {
        let engine = Arc::new(ExecutionEngine::new());
        let minted_id = Arc::new(StdMutex::new(None));
        let release = Arc::new(Notify::new());
        let remote = BlockingMintCapturingRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(tucana::shared::node_execution_result::Id::NodeId(1)),
                result: Some(tucana::shared::node_execution_result::Result::Success(
                    int_value(1),
                )),
            },
            minted_id: Arc::clone(&minted_id),
            release: Arc::clone(&release),
        };

        let remote_node = node(
            1,
            "remote::open_stream",
            vec![sub_flow_param(100, "on_message", 2)],
            None,
            Some("action.svc"),
        );
        // Reads both action-supplied arguments back independently via their
        // own `input_index`, so the assertion below proves neither slot
        // leaked into or overwrote the other.
        let sub_flow_target = node(
            2,
            "std::number::add",
            vec![
                input_type_param(200, "first", 1, 0, 0),
                input_type_param(201, "second", 1, 0, 1),
            ],
            None,
            None,
        );

        let flow = ExecutionFlow {
            flow_id: 1,
            project_id: 7,
            starting_node_id: 1,
            node_functions: vec![remote_node, sub_flow_target],
            input_value: None,
        };

        let parent_engine = Arc::clone(&engine);
        let parent = tokio::spawn(async move {
            parent_engine
                .execute_flow_report_async("parent-1", flow, Some(&remote), false)
                .await
        });

        let execution_identifier = wait_for_minted_id(&minted_id).await;

        let request = ActionSubFlowExecutionRequest {
            execution_identifier: execution_identifier.clone(),
            parameters: vec![int_value(3), int_value(4)],
        };
        let result = build_sub_flow_execution_result(request, &engine, None, false).await;
        match result.result {
            Some(execution_result::Result::Success(value)) => assert_eq!(value, int_value(7)),
            other => panic!("expected success result, got {:?}", other),
        }

        release.notify_one();
        parent.await.expect("parent task should not panic");
    }

    #[tokio::test]
    async fn sub_flow_execution_result_reports_not_found_for_unknown_id() {
        let engine = ExecutionEngine::new();
        let request = ActionSubFlowExecutionRequest {
            execution_identifier: "does-not-exist".to_string(),
            parameters: Vec::new(),
        };

        let result = build_sub_flow_execution_result(request, &engine, None, false).await;

        assert_eq!(result.execution_identifier, "does-not-exist");
        match result.result {
            Some(execution_result::Result::Error(err)) => {
                assert_eq!(err.code, "T-TAURUS-000002");
                assert_eq!(err.category, "SubFlowExecutionNotFound");
            }
            other => panic!("expected not-found error result, got {:?}", other),
        }
    }

    #[test]
    fn sub_flow_decode_error_result_reports_expected_code() {
        let result = build_sub_flow_decode_error_result();

        match result.result {
            Some(execution_result::Result::Error(err)) => {
                assert_eq!(err.code, "T-TAURUS-000003");
                assert_eq!(err.category, "SubFlowExecutionRequestDecodeError");
            }
            other => panic!("expected decode error result, got {:?}", other),
        }
    }
}
