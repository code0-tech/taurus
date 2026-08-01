//! Streams execution lifecycle events (`starting`/`ongoing`/`finished`/`failed`)
//! to `<topic_prefix>.<execution_id>` on NATS. See [`NATSRespondEmitter`] for
//! why publishing happens on a background task rather than inline in `emit`.

use async_nats::Client;
use prost::Message;
use std::collections::HashMap;
use taurus_core::runtime::engine::{EmitType, ExecutionId, RespondEmitter};
use tokio::sync::mpsc;
use tucana::shared::value::Kind::{StringValue, StructValue};
use tucana::shared::{Struct, Value};

const DEFAULT_TOPIC_PREFIX: &str = "runtime.emitter";

/// Keeps the synchronous `RespondEmitter::emit` API on the hot execution
/// path non-blocking by handing events to a background task over a bounded
/// channel; that task does the actual NATS publish. The channel is bounded
/// (rather than unbounded) because with concurrent flow execution, many
/// tasks can call `emit` in a burst while NATS publishing is comparatively
/// slow -- an unbounded queue would grow without limit if publishing ever
/// lags. Once full, `emit` drops the event rather than blocking the
/// caller; these are best-effort lifecycle notifications, not the
/// authoritative execution result (that goes back via gRPC separately).
/// Call [`NATSRespondEmitter::shutdown`] to drain queued events before
/// dropping this, since an aborted owner leaves them unpublished.
pub struct NATSRespondEmitter {
    tx: mpsc::Sender<NATSEmitMessage>,
    worker_task: tokio::task::JoinHandle<()>,
}

struct NATSEmitMessage {
    execution_id: ExecutionId,
    emit_type: EmitType,
    value: Value,
}

impl NATSRespondEmitter {
    pub fn new(client: Client, channel_capacity: usize) -> Self {
        Self::with_topic_prefix(client, DEFAULT_TOPIC_PREFIX, channel_capacity)
    }

    pub fn with_topic_prefix(
        client: Client,
        topic_prefix: impl Into<String>,
        channel_capacity: usize,
    ) -> Self {
        let topic_prefix = topic_prefix.into();
        let (tx, mut rx) = mpsc::channel::<NATSEmitMessage>(channel_capacity.max(1));

        // Keep the public emitter API synchronous while publishing asynchronously.
        // This worker serializes outbound lifecycle events to one NATS topic per execution:
        // `<topic_prefix>.<execution_id>`.
        // Event type is embedded in the payload so subscribers do not need four topic bindings.
        let worker_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                log::debug!(
                    "Emitter has been called for Emitter Signal: {}",
                    message.emit_type
                );
                let topic = format!("{}.{}", topic_prefix, message.execution_id);
                let encoded_payload = encode_emit_message(message.emit_type, message.value);

                if let Err(err) = client
                    .publish(topic.clone(), encoded_payload.encode_to_vec().into())
                    .await
                {
                    log::error!(
                        "Failed to publish runtime emit message on '{}': {:?}",
                        topic,
                        err
                    );
                } else {
                    log::info!(
                        "Published runtime emit message on '{}' (type={})",
                        topic,
                        emit_type_as_str(message.emit_type)
                    );
                }
            }
        });

        Self { tx, worker_task }
    }

    /// Gracefully stop the emitter worker after all queued messages are published.
    pub async fn shutdown(self) {
        drop(self.tx);
        if let Err(err) = self.worker_task.await {
            log::error!("NATS emitter worker join failed: {:?}", err);
        }
    }
}

impl RespondEmitter for NATSRespondEmitter {
    fn emit(&self, execution_id: ExecutionId, emit_type: EmitType, value: Value) {
        match self.tx.try_send(NATSEmitMessage {
            execution_id,
            emit_type,
            value,
        }) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                log::warn!(
                    "Dropped runtime emit message for execution {}: emitter channel full \
                     (publishing is lagging behind emit rate)",
                    execution_id
                );
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                log::debug!(
                    "Dropped runtime emit message for execution {}: emitter worker is unavailable",
                    execution_id
                );
            }
        }
    }
}

fn encode_emit_message(emit_type: EmitType, payload: Value) -> Value {
    let emit_type_value = Value {
        kind: Some(StringValue(emit_type_as_str(emit_type).to_string())),
    };

    Value {
        kind: Some(StructValue(Struct {
            fields: HashMap::from([
                ("emit_type".to_string(), emit_type_value),
                ("payload".to_string(), payload),
            ]),
        })),
    }
}

fn emit_type_as_str(emit_type: EmitType) -> &'static str {
    match emit_type {
        EmitType::StartingExec => "starting",
        EmitType::OngoingExec => "ongoing",
        EmitType::FinishedExec => "finished",
        EmitType::FailedExec => "failed",
    }
}
