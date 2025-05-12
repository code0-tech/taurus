pub mod context;
pub mod error;
pub mod implementation;
pub mod locale;
pub mod registry;

use std::sync::Arc;

use code0_flow::flow_queue::service::{Message, RabbitmqClient};
use context::Context;
use error::RuntimeError;
use futures_lite::StreamExt;
use lapin::{options::BasicConsumeOptions, types::FieldTable};
use locale::locale::Locale;
use registry::FunctionStore;
use tucana::shared::{Flow, NodeFunction, Value};

fn handle_node_function(
    function: NodeFunction,
    store: &FunctionStore,
    context: &mut Context,
) -> Result<Value, RuntimeError> {
    if let Some(definition) = function.definition {
        let runtime_function = match store.get(definition.runtime_function_id.as_str()) {
            Some(fc) => fc,
            None => todo!("Retrun if no funtion is present"),
        };

        let mut parameter_collection: Vec<Value> = vec![];

        for parameter in function.parameters {
            if let Some(value) = parameter.value {
                match value {
                    // Its just a normal value, directly a paramter
                    tucana::shared::node_parameter::Value::LiteralValue(v) => {
                        parameter_collection.push(v)
                    }

                    // Its a reference to an already executed function that returns value is the parameter of this function
                    tucana::shared::node_parameter::Value::ReferenceValue(reference) => {
                        let optional_value = context.get(&reference);

                        // Look if its even present
                        let context_result = match optional_value {
                            Some(context_result) => context_result,
                            None => {
                                todo!("Required function that holds the parameter wasnt executed")
                            }
                        };

                        match context_result {
                            Ok(v) => {
                                parameter_collection.push(v.clone());
                            }
                            Err(_) => {
                                todo!(
                                    "Reqired function that holds the paramter failed in execution"
                                )
                            }
                        }
                    }

                    // Its another function, that result is a direct parameter to this function
                    tucana::shared::node_parameter::Value::FunctionValue(another_node_function) => {
                        let function_result =
                            handle_node_function(another_node_function, &store, context);

                        match function_result {
                            Ok(v) => {
                                parameter_collection.push(v.clone());
                            }
                            Err(_) => {
                                todo!(
                                    "Reqired function that holds the paramter failed in execution"
                                )
                            }
                        }
                    }
                }
            }
        }

        let result = runtime_function(&parameter_collection, context);

        if let Some(ref next_node) = function.next_node {
            let next: NodeFunction = (**next_node).clone();
            let _ = handle_node_function(next, store, context);
            todo!()
        };

        return result;
    };

    Err(RuntimeError::default())
}

fn handle_message(
    message: Message,
    store: &FunctionStore,
    context: &mut Context,
) -> Result<Message, lapin::Error> {
    let flow: Flow = match serde_json::from_str(&message.body) {
        Ok(flow) => flow,
        Err(_) => {
            todo!()
        }
    };

    if let Some(node) = flow.starting_node {
        match handle_node_function(node, store, context) {
            Ok(result) => match serde_json::to_string(&result) {
                Ok(res) => {
                    return Ok(Message {
                        message_id: message.message_id,
                        message_type: message.message_type,
                        timestamp: message.timestamp,
                        sender: message.sender,
                        body: res,
                    })
                }
                Err(_) => {
                    todo!("")
                }
            },
            Err(runtime_error) => {
                return Ok(Message {
                    message_id: message.message_id,
                    message_type: message.message_type,
                    timestamp: message.timestamp,
                    sender: message.sender,
                    body: runtime_error.to_string(),
                })
            }
        }
    };

    Ok(Message {
        message_id: message.message_id,
        message_type: message.message_type,
        timestamp: message.timestamp,
        sender: message.sender,
        body: "{ \"text\": \"Hihi, World!\" }".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let _locale = Locale::default();
    let store = FunctionStore::new();
    let mut context = Context::new();

    let rabbitmq_client = Arc::new(RabbitmqClient::new("amqp://localhost:5672").await);

    let mut consumer = {
        let channel = rabbitmq_client.channel.lock().await;

        let consumer_res = channel
            .basic_consume(
                "send_queue",
                "consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await;

        match consumer_res {
            Ok(consumer) => consumer,
            Err(err) => panic!("Cannot consume messages: {}", err),
        }
    };

    log::debug!("Starting to consume from send_queue");

    while let Some(delivery) = consumer.next().await {
        let delivery = match delivery {
            Ok(del) => del,
            Err(err) => {
                log::error!("Error receiving message: {}", err);
                return;
            }
        };

        let data = &delivery.data;
        let message_str = match std::str::from_utf8(&data) {
            Ok(str) => {
                log::info!("Received message: {}", str);
                str
            }
            Err(err) => {
                log::error!("Error decoding message: {}", err);
                return;
            }
        };
        // Parse the messagey
        let inc_message = match serde_json::from_str::<Message>(message_str) {
            Ok(mess) => mess,
            Err(err) => {
                log::error!("Error parsing message: {}", err);
                return;
            }
        };

        let message = match handle_message(inc_message, &store, &mut context) {
            Ok(mess) => mess,
            Err(err) => {
                log::error!("Error handling message: {}", err);
                return;
            }
        };

        let message_json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(err) => {
                log::error!("Error serializing message: {}", err);
                return;
            }
        };

        {
            let _ = rabbitmq_client
                .send_message(message_json, "recieve_queue")
                .await;
        }

        // Acknowledge the message
        delivery
            .ack(lapin::options::BasicAckOptions::default())
            .await
            .expect("Failed to acknowledge message");
    }
}
