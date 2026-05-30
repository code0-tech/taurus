use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::engine::{EmitType, ExecutionEngine, ExecutionId, RespondEmitter};
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::task::JoinHandle;
use tucana::shared::execution_result;
use tucana::shared::{ExecutionFlow, ExecutionResult, RuntimeUsage, Value};

use crate::client::runtime_usage::TaurusRuntimeUsageService;

pub fn spawn_worker(
    client: async_nats::Client,
    engine: ExecutionEngine,
    nats_remote: NATSRemoteRuntime,
    runtime_emitter: NATSRespondEmitter,
    runtime_usage_service: Option<TaurusRuntimeUsageService>,
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

        let mut test_execution_subscription = match client
            .queue_subscribe(String::from("test_executions.*"), "taurus".into())
            .await
        {
            Ok(subscription) => {
                log::info!("Subscribed to 'test_executions.*'");
                subscription
            }
            Err(err) => {
                log::error!("Failed to subscribe to 'test_executions.*': {:?}", err);
                return;
            }
        };

        let mut execution_closed = false;
        let mut test_execution_closed = false;

        while !(execution_closed && test_execution_closed) {
            tokio::select! {
                message = execution_subscription.next(), if !execution_closed => {
                    match message {
                        Some(message) => {
                            process_execution_message(
                                message,
                                &engine,
                                &nats_remote,
                                &runtime_emitter,
                                runtime_usage_service.as_ref(),
                            ).await;
                        }
                        None => {
                            execution_closed = true;
                            log::warn!("Subscription 'execution.*' ended");
                        }
                    }
                }
                message = test_execution_subscription.next(), if !test_execution_closed => {
                    match message {
                        Some(message) => {
                            process_test_execution_message(
                                &client,
                                message,
                                &engine,
                                &nats_remote,
                                runtime_usage_service.as_ref(),
                            ).await;
                        }
                        None => {
                            test_execution_closed = true;
                            log::warn!("Subscription 'test_executions.*' ended");
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
    runtime_usage_service: Option<&TaurusRuntimeUsageService>,
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
        nats_remote,
        Some(&respond_emitter),
    );
    log::debug!(
        "Flow {} execution completed; no direct reply message published",
        flow_id
    );

    if let Some(usage_service) = runtime_usage_service {
        usage_service
            .update_runtime_usage(run_result.runtime_usage)
            .await;
    }
}

async fn process_test_execution_message(
    client: &async_nats::Client,
    message: async_nats::Message,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    runtime_usage_service: Option<&TaurusRuntimeUsageService>,
) {
    let requested_execution_id =
        match parse_execution_id_from_subject(&message.subject, "test_executions") {
            Some(res) => res,
            None => {
                log::error!("Failed to extract execution uuid from {}", &message.subject);
                return;
            }
        };

    let flow: ExecutionFlow = match ExecutionFlow::decode(&*message.payload) {
        Ok(flow) => flow,
        Err(err) => {
            log::error!(
                "Failed to deserialize test execution flow: {:?}, payload: {:?}",
                err,
                &message.payload
            );
            let result = build_decode_error_result(requested_execution_id);
            respond_to_test_execution_request(client, &message, result).await;
            return;
        }
    };

    let run_result = execute_flow(requested_execution_id, flow, engine, nats_remote, None);

    if let Some(usage_service) = runtime_usage_service {
        usage_service
            .update_runtime_usage(run_result.runtime_usage.clone())
            .await;
    }

    let execution_result = build_execution_result(
        run_result.execution_id,
        run_result.flow_id,
        run_result.started_at,
        run_result.finished_at,
        run_result.input,
        run_result.signal,
    );
    respond_to_test_execution_request(client, &message, execution_result).await;
}

#[derive(Clone)]
struct FlowRunResult {
    execution_id: ExecutionId,
    flow_id: i64,
    started_at: i64,
    finished_at: i64,
    input: Option<Value>,
    signal: Signal,
    runtime_usage: RuntimeUsage,
}

fn execute_flow(
    execution_id: ExecutionId,
    flow: ExecutionFlow,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    respond_emitter: Option<&dyn RespondEmitter>,
) -> FlowRunResult {
    let started_at = now_unix_ms();
    let start = Instant::now();
    let flow_id = flow.flow_id;
    let input = flow.input_value.clone();
    let (signal, _reason) = engine.execute_flow_with_execution_id(
        execution_id,
        flow,
        Some(nats_remote),
        respond_emitter,
        true,
    );
    let finished_at = now_unix_ms();
    let duration_millis = start.elapsed().as_millis() as i64;

    FlowRunResult {
        execution_id,
        flow_id,
        started_at,
        finished_at,
        input,
        signal,
        runtime_usage: RuntimeUsage {
            flow_id,
            duration: duration_millis,
        },
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
        node_execution_results: Vec::new(),
        result,
    }
}

fn build_decode_error_result(execution_id: ExecutionId) -> ExecutionResult {
    let now = now_unix_ms();
    let runtime_error = RuntimeError::new(
        "T-TAURUS-000001",
        "ExecutionFlowDecodeError",
        "Failed to decode test execution flow payload",
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

async fn respond_to_test_execution_request(
    client: &async_nats::Client,
    message: &async_nats::Message,
    result: ExecutionResult,
) {
    let Some(reply_subject) = message.reply.as_ref() else {
        log::warn!(
            "Received test execution request without reply subject on '{}'; cannot return ExecutionResult",
            message.subject
        );
        return;
    };

    if let Err(err) = client
        .publish(reply_subject.clone(), result.encode_to_vec().into())
        .await
    {
        log::error!(
            "Failed to publish test execution response on '{}': {:?}",
            reply_subject,
            err
        );
    }
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|it| it.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    use prost::Message;
    use serde::Deserialize;
    use taurus_core::runtime::engine::ExecutionEngine;
    use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
    use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
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

    static NATS_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    #[ignore = "requires a running NATS server at NATS_URL or nats://127.0.0.1:4222"]
    fn test_execution_request_returns_execution_result_over_nats() {
        let _lock = NATS_TEST_LOCK
            .lock()
            .expect("NATS test lock should not be poisoned");
        runtime().block_on(async {
            let client = connect_test_nats().await;
            let worker = spawn_test_worker(client.clone());
            wait_for_worker_subscription().await;

            let execution_id = ExecutionId::new_v4();
            let fixture = load_fixture("flows/01_return_object.json");
            let expected_result = fixture.inputs[0].expected_result.clone();
            let flow = execution_flow_from_fixture(fixture);
            let response = request_execution_result(&client, execution_id, flow.encode_to_vec())
                .await
                .expect("test execution request should receive an ExecutionResult response");

            worker.abort();

            assert_eq!(response.execution_identifier, execution_id.to_string());
            assert_eq!(response.flow_id, flow.flow_id);
            assert!(response.started_at > 0);
            assert!(response.finished_at >= response.started_at);
            assert_eq!(response.input, None);
            assert!(response.node_execution_results.is_empty());

            match response.result {
                Some(execution_result::Result::Success(value)) => {
                    assert_eq!(to_json_value(value), expected_result);
                }
                other => panic!("expected successful test execution result, got {:?}", other),
            }
        });
    }

    #[test]
    #[ignore = "requires a running NATS server at NATS_URL or nats://127.0.0.1:4222"]
    fn test_execution_request_returns_decode_error_over_nats() {
        let _lock = NATS_TEST_LOCK
            .lock()
            .expect("NATS test lock should not be poisoned");
        runtime().block_on(async {
            let client = connect_test_nats().await;
            let worker = spawn_test_worker(client.clone());
            wait_for_worker_subscription().await;

            let execution_id = ExecutionId::new_v4();
            let response =
                request_execution_result(&client, execution_id, b"not protobuf".to_vec())
                    .await
                    .expect("malformed test execution request should receive an error response");

            worker.abort();

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
                    assert_eq!(
                        error.message,
                        "Failed to decode test execution flow payload"
                    );
                }
                other => panic!("expected decode error result, got {:?}", other),
            }
        });
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test runtime should build")
    }

    async fn connect_test_nats() -> async_nats::Client {
        let nats_url =
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
        async_nats::connect(&nats_url)
            .await
            .unwrap_or_else(|err| panic!("failed to connect to NATS at {nats_url}: {err}"))
    }

    fn spawn_test_worker(client: async_nats::Client) -> tokio::task::JoinHandle<()> {
        let engine = ExecutionEngine::new();
        let nats_remote = NATSRemoteRuntime::new(client.clone());
        let runtime_emitter = NATSRespondEmitter::new(client.clone());
        spawn_worker(client, engine, nats_remote, runtime_emitter, None)
    }

    async fn wait_for_worker_subscription() {
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    async fn request_execution_result(
        client: &async_nats::Client,
        execution_id: ExecutionId,
        payload: Vec<u8>,
    ) -> Result<ExecutionResult, String> {
        let subject = format!("test_executions.{execution_id}");

        for attempt in 1..=10 {
            match tokio::time::timeout(
                Duration::from_secs(2),
                client.request(subject.clone(), payload.clone().into()),
            )
            .await
            {
                Ok(Ok(message)) => {
                    return ExecutionResult::decode(&*message.payload)
                        .map_err(|err| format!("failed to decode ExecutionResult: {err}"));
                }
                Ok(Err(err)) if attempt < 10 => {
                    log::debug!(
                        "test execution request attempt {} failed before subscription was ready: {:?}",
                        attempt,
                        err
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Ok(Err(err)) => return Err(format!("NATS request failed: {err}")),
                Err(_) if attempt < 10 => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(_) => return Err("timed out waiting for test execution response".to_string()),
            }
        }

        Err("test execution request did not complete".to_string())
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
