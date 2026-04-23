use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use futures_lite::StreamExt;
use prost::Message;
use taurus_core::runtime::engine::{EmitType, ExecutionEngine, RespondEmitter};
use taurus_core::types::signal::Signal;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::task::JoinHandle;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, RuntimeUsage, Value};

use crate::client::runtime_usage::TaurusRuntimeUsageService;

pub fn spawn_worker(
    client: async_nats::Client,
    engine: ExecutionEngine,
    nats_remote: NATSRemoteRuntime,
    runtime_usage_service: Option<TaurusRuntimeUsageService>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let runtime_emitter = NATSRespondEmitter::new(client.clone());

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
                &client,
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
    client: &async_nats::Client,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    runtime_emitter: &NATSRespondEmitter,
    runtime_usage_service: Option<&TaurusRuntimeUsageService>,
) {
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
    let reply_subject = message.reply.clone();
    let respond_count = Arc::new(AtomicUsize::new(0));
    let respond_count_for_emitter = respond_count.clone();
    let respond_emitter = |execution_id, emit_type: EmitType, value: Value| {
        match emit_type {
            EmitType::OngoingExec => {
                respond_count_for_emitter.fetch_add(1, Ordering::Relaxed);
            }
            EmitType::StartingExec => log::debug!("Flow execution started"),
            EmitType::FinishedExec => log::debug!("Flow execution finished"),
            EmitType::FailedExec => log::debug!("Flow execution failed"),
        }
        runtime_emitter.emit(execution_id, emit_type, value);
    };
    let (signal, runtime_usage) = execute_flow(flow, engine, nats_remote, Some(&respond_emitter));

    let has_responded = respond_count.load(Ordering::Relaxed) > 0;
    let final_value = signal_to_terminal_value(signal);

    // Stream contract: if we already emitted intermediate values, do not send an extra terminal
    // payload on the same reply subject.
    if let Some(value) = final_value
        && !has_responded
        && let Some(reply_subject) = reply_subject
    {
        log::info!("Returning value for flow_id {}: {:?}", flow_id, value);
        if let Err(err) = client
            .publish(reply_subject, value.encode_to_vec().into())
            .await
        {
            log::error!("Failed to send response: {:?}", err);
        }
    }

    if let Some(usage_service) = runtime_usage_service {
        usage_service.update_runtime_usage(runtime_usage).await;
    }
}

fn execute_flow(
    flow: ExecutionFlow,
    engine: &ExecutionEngine,
    nats_remote: &NATSRemoteRuntime,
    respond_emitter: Option<&dyn RespondEmitter>,
) -> (Signal, RuntimeUsage) {
    let start = Instant::now();
    let flow_id = flow.flow_id;
    let (signal, _reason) = engine.execute_flow(flow, Some(nats_remote), respond_emitter, true);
    let duration_millis = start.elapsed().as_millis() as i64;

    (
        signal,
        RuntimeUsage {
            flow_id,
            duration: duration_millis,
        },
    )
}

fn signal_to_terminal_value(signal: Signal) -> Option<Value> {
    match signal {
        Signal::Failure(error) => {
            log::error!("Runtime error occurred: {:?}", error);
            Some(error.as_value())
        }
        Signal::Success(value) => {
            log::debug!("Execution ended with success signal");
            Some(value)
        }
        Signal::Return(value) => {
            log::debug!("Execution ended with return signal");
            Some(value)
        }
        Signal::Respond(_) => {
            log::debug!("Execution ended with respond signal");
            None
        }
        Signal::Stop => {
            log::debug!("Received stop signal as last signal");
            Some(Value {
                kind: Some(Kind::NullValue(0)),
            })
        }
    }
}
