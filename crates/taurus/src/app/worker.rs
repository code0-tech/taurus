use std::time::Instant;

use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::engine::{EmitType, ExecutionEngine, ExecutionId, RespondEmitter};
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::task::JoinHandle;
use tucana::shared::{ExecutionFlow, RuntimeUsage, Value};

use crate::client::runtime_usage::TaurusRuntimeUsageService;

pub fn spawn_worker(
    client: async_nats::Client,
    engine: ExecutionEngine,
    nats_remote: NATSRemoteRuntime,
    runtime_emitter: NATSRespondEmitter,
    runtime_usage_service: Option<TaurusRuntimeUsageService>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut subscription = match client
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

        while let Some(message) = subscription.next().await {
            process_message(
                message,
                &engine,
                &nats_remote,
                &runtime_emitter,
                runtime_usage_service.as_ref(),
            )
            .await;
        }

        log::info!("NATS worker loop ended");
    })
}

async fn process_message(
    message: async_nats::Message,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    runtime_emitter: &NATSRespondEmitter,
    runtime_usage_service: Option<&TaurusRuntimeUsageService>,
) {
    let requested_execution_id =
        parse_execution_id_from_subject(&message.subject).unwrap_or_else(|| {
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
    let runtime_usage = execute_flow(
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
        usage_service.update_runtime_usage(runtime_usage).await;
    }
}

fn execute_flow(
    execution_id: ExecutionId,
    flow: ExecutionFlow,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    respond_emitter: Option<&dyn RespondEmitter>,
) -> RuntimeUsage {
    let start = Instant::now();
    let flow_id = flow.flow_id;
    let (_signal, _reason) = engine.execute_flow_with_execution_id(
        execution_id,
        flow,
        Some(nats_remote),
        respond_emitter,
        true,
    );
    let duration_millis = start.elapsed().as_millis() as i64;

    RuntimeUsage {
        flow_id,
        duration: duration_millis,
    }
}

fn parse_execution_id_from_subject(subject: &async_nats::Subject) -> Option<ExecutionId> {
    let raw = subject.as_str();
    let mut parts = raw.split('.');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("execution"), Some(uuid), None) => ExecutionId::parse_str(uuid).ok(),
        _ => None,
    }
}
