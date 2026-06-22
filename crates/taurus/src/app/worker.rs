use std::time::Instant;

use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::engine::{EmitType, ExecutionEngine, ExecutionId, RespondEmitter};
use taurus_core::runtime::remote::RemoteRuntime;
use taurus_core::time::now_unix_micros;
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::task::JoinHandle;
use tucana::shared::execution_result;
use tucana::shared::{ExecutionFlow, ExecutionResult, NodeExecutionResult, Value};

use crate::client::runtime_execution::TaurusRuntimeExecutionService;

pub fn spawn_worker(
    client: async_nats::Client,
    engine: ExecutionEngine,
    nats_remote: NATSRemoteRuntime,
    runtime_emitter: NATSRespondEmitter,
    mut runtime_execution_service: Option<TaurusRuntimeExecutionService>,
) -> JoinHandle<()> {
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
                return;
            }
        };

        let mut execution_closed = false;

        while !execution_closed {
            tokio::select! {
                message = execution_subscription.next(), if !execution_closed => {
                    match message {
                        Some(message) => {
                            process_execution_message(
                                message,
                                &engine,
                                &nats_remote,
                                &runtime_emitter,
                                runtime_execution_service.as_mut(),
                            ).await;
                        }
                        None => {
                            execution_closed = true;
                            log::warn!("Subscription 'execution.*' ended");
                        }
                    }
                }
            }
        }

        log::info!("NATS worker loop ended");
    })
}

async fn process_execution_message(
    message: async_nats::Message,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    runtime_emitter: &NATSRespondEmitter,
    mut runtime_execution_service: Option<&mut TaurusRuntimeExecutionService>,
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
            if let Some(execution_service) = runtime_execution_service.as_mut() {
                execution_service
                    .update_runtime_execution(build_decode_error_result(requested_execution_id))
                    .await;
            }
            return;
        }
    };

    let flow_id = flow.flow_id;
    // Taurus app forwards all lifecycle events to emitter.
    // Direct request/reply responses remain disabled; delivery is emitter-only.
    let respond_emitter = |execution_id, emit_type: EmitType, value: Value| {
        runtime_emitter.emit(execution_id, emit_type, value);
    };
    let run_result = execute_flow(
        requested_execution_id,
        flow,
        engine,
        Some(nats_remote),
        Some(&respond_emitter),
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
    respond_emitter: Option<&dyn RespondEmitter>,
) -> FlowRunResult {
    let started_at = now_unix_micros();
    let flow_id = flow.flow_id;
    let input = flow.input_value.clone();
    let report = engine
        .execute_flow_with_execution_id_report_async(
            execution_id,
            flow,
            remote,
            respond_emitter,
            true,
        )
        .await;
    let finished_at = now_unix_micros();

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
        Signal::Success(value) | Signal::Return(value) | Signal::Respond(value) => {
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

    use serde::Deserialize;
    use tucana::shared::{
        ValidationFlow, execution_result,
        helper::value::{from_json_value, to_json_value},
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
        let fixture = load_fixture("flows/01_return_object.json");
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
        let fixture = load_fixture("flows/01_return_object.json");
        let flow = execution_flow_from_fixture(fixture);
        let engine = ExecutionEngine::new();

        let run_result = execute_flow(execution_id, flow, &engine, None, None).await;

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
}
