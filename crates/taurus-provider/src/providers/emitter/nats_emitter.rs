use async_nats::Client;
use prost::Message;
use std::collections::HashMap;
use taurus_core::runtime::engine::{EmitType, ExecutionId, RespondEmitter};
use tokio::sync::mpsc;
use tucana::shared::value::Kind::{StringValue, StructValue};
use tucana::shared::{Struct, Value};

const DEFAULT_TOPIC_PREFIX: &str = "runtime.emitter";

pub struct NATSRespondEmitter {
    tx: mpsc::UnboundedSender<NATSEmitMessage>,
}

struct NATSEmitMessage {
    execution_id: ExecutionId,
    emit_type: EmitType,
    value: Value,
}

impl NATSRespondEmitter {
    pub fn new(client: Client) -> Self {
        Self::with_topic_prefix(client, DEFAULT_TOPIC_PREFIX)
    }

    pub fn with_topic_prefix(client: Client, topic_prefix: impl Into<String>) -> Self {
        let topic_prefix = topic_prefix.into();
        let (tx, mut rx) = mpsc::unbounded_channel::<NATSEmitMessage>();

        // Keep the public emitter API synchronous while publishing asynchronously.
        // This worker serializes outbound lifecycle events to one NATS topic per execution:
        // `<topic_prefix>.<execution_id>`.
        // Event type is embedded in the payload so subscribers do not need four topic bindings.
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
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
                }
            }
        });

        Self { tx }
    }
}

impl RespondEmitter for NATSRespondEmitter {
    fn emit(&self, execution_id: ExecutionId, emit_type: EmitType, value: Value) {
        if let Err(err) = self.tx.send(NATSEmitMessage {
            execution_id,
            emit_type,
            value,
        }) {
            log::debug!(
                "Dropped runtime emit message because NATS emitter worker is unavailable: {:?}",
                err
            );
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
