mod configuration;
pub mod context;
pub mod error;
pub mod implementation;
pub mod locale;
pub mod registry;
use std::sync::Arc;

use code0_flow::{
    flow_config::load_env_file,
    flow_queue::service::{Message, RabbitmqClient},
};
use context::{Context, ContextEntry, ContextResult};
use error::RuntimeError;
use futures_lite::StreamExt;
use lapin::{options::BasicConsumeOptions, types::FieldTable};
use locale::locale::Locale;
use registry::FunctionStore;
use tucana::shared::{Flow, NodeFunction, Value};

use crate::configuration::Config;

fn handle_node_function(
    function: NodeFunction,
    store: &FunctionStore,
    context: &mut Context,
) -> Result<Value, RuntimeError> {
    let runtime_function = match store.get(function.runtime_function_id.as_str()) {
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

                    // A reference is present. Look up the real value
                    match context_result {
                        // The reference is a exeuction result of a node
                        ContextResult::NodeExecutionResult(node_result) => match node_result {
                            Ok(value) => {
                                parameter_collection.push(value.clone());
                            }
                            Err(err) => return Err(err),
                        },
                        // The reference is a parameter of a node
                        ContextResult::ParameterResult(parameter_result) => {
                            parameter_collection.push(parameter_result.clone());
                        }
                    }
                }

                // Its another function, that result is a direct parameter to this function
                tucana::shared::node_parameter::Value::FunctionValue(another_node_function) => {
                    // As this is another new indent, a new context will be opened
                    context.next_context();
                    let function_result =
                        handle_node_function(another_node_function, &store, context);

                    let entry =
                        ContextEntry::new(function_result.clone(), parameter_collection.clone());
                    context.write_to_current_context(entry);

                    match function_result {
                        Ok(v) => {
                            // Add the value back to the main parameter
                            parameter_collection.push(v.clone());
                        }
                        Err(_) => {
                            todo!("Reqired function that holds the paramter failed in execution")
                        }
                    }
                }
            }
        }

        let result = runtime_function(&parameter_collection, context);

        // Result will be added to the current context
        let entry = ContextEntry::new(result.clone(), parameter_collection.clone());
        context.write_to_current_context(entry);

        // Check if there is a next node, if not then this was the last one
        match function.next_node {
            Some(ref next_node_function) => {
                let next = (**next_node_function).clone();

                // Increment the context node!
                context.next_node();

                return handle_node_function(next, store, context);
            }
            None => {
                if context.is_end() {
                    return result;
                }

                context.leave_context();
            }
        }
    }

    Err(RuntimeError::default())
}

fn handle_message(message: Message, store: &FunctionStore) -> Result<Message, lapin::Error> {
    let mut context = Context::new();

    let flow: Flow = match serde_json::from_str(&message.body) {
        Ok(flow) => flow,
        Err(_) => {
            todo!()
        }
    };

    if let Some(node) = flow.starting_node {
        match handle_node_function(node, store, &mut context) {
            Ok(result) => match serde_json::to_string(&result) {
                Ok(res) => {
                    return Ok(Message {
                        message_id: message.message_id,
                        message_type: message.message_type,
                        timestamp: message.timestamp,
                        sender: message.sender,
                        body: res,
                    });
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
                });
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    load_env_file();

    let config = Config::new();

    let _locale = Locale::default();
    let store = FunctionStore::new();

    let rabbitmq_client = Arc::new(RabbitmqClient::new(config.rabbitmq_url.as_str()).await);

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

        let message = match handle_message(inc_message, &store) {
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
