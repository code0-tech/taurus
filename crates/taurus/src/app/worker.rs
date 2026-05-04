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

