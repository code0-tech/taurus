pub mod functions;

use code0_flow::flow_queue::service::{Message, RabbitmqClient};
use std::sync::Arc;
use tucana::shared::Flow;

fn handle_message(message: Message) -> Result<Message, lapin::Error> {
    let s = serde_json::from_str::<String>(&message.body).unwrap();
    let flow = match serde_json::from_str::<Flow>(&s) {
        Ok(flow) => flow,
        Err(e) => {
            eprintln!("Failed to deserialize flow: {}", e);
            todo!();
        }
    };

    match flow.starting_node {
        Some(node) => {
            let function_definiton = match node.definition {
                Some(f) => f,
                None => todo!(),
            };

            let operation = match functions::std::math::MathExpression::from_string(
                function_definiton.runtime_function_id.as_str(),
            ) {
                Some(op) => op,
                None => {
                    todo!("{}", function_definiton.runtime_function_id.as_str());
                }
            };
            let first = match node.parameters.get(0) {
                Some(param) => match param.value.as_ref() {
                    Some(tucana::shared::node_parameter::Value::LiteralValue(value)) => {
                        if let Some(tucana::shared::value::Kind::NumberValue(number)) = &value.kind
                        {
                            *number
                        } else {
                            todo!();
                        }
                    }
                    _ => todo!(),
                },
                None => todo!(),
            };

            let second = match node.parameters.get(1) {
                Some(param) => match param.value.as_ref() {
                    Some(tucana::shared::node_parameter::Value::LiteralValue(value)) => {
                        if let Some(tucana::shared::value::Kind::NumberValue(number)) = &value.kind
                        {
                            *number
                        } else {
                            todo!();
                        }
                    }
                    _ => todo!(),
                },
                None => todo!(),
            };

            let result = operation.evaluate(first, second);

            return Ok(Message {
                message_id: message.message_id,
                message_type: message.message_type,
                timestamp: message.timestamp,
                sender: message.sender,
                body: format!("{{ \"result\": {} }}", result),
            });
        }
        None => todo!(),
    };
}

#[tokio::main]
async fn main() {
    let rabbitmq_client = Arc::new(RabbitmqClient::new("amqp://localhost:5672").await);

    // Receive messages from the send_queue
    if let Err(e) = rabbitmq_client
        .receive_messages("send_queue", handle_message)
        .await
    {
        eprintln!("Failed to receive messages: {}", e);
    }
}
