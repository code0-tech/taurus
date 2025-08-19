mod configuration;
pub mod context;
pub mod error;
pub mod implementation;
pub mod registry;

use crate::configuration::Config;
use crate::implementation::collect;
use code0_flow::flow_config::load_env_file;
use context::{Context, ContextEntry, ContextResult};
use error::RuntimeError;
use futures_lite::StreamExt;
use prost::Message;
use registry::FunctionStore;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, ListValue, NodeFunction, Value};

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
        if let Some(node_value) = parameter.value {
            if let Some(value) = node_value.value {
                match value {
                    // Its just a normal value, directly a paramter
                    tucana::shared::node_value::Value::LiteralValue(v) => {
                        parameter_collection.push(v)
                    }
                    // Its a reference to an already executed function that returns value is the parameter of this function
                    tucana::shared::node_value::Value::ReferenceValue(reference) => {
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
                    tucana::shared::node_value::Value::NodeFunctions(another_node_function) => {
                        // As this is another new indent, a new context will be opened
                        context.next_context();
                        let function_result: Vec<_> = another_node_function
                            .functions
                            .into_iter()
                            .map(|f| handle_node_function(f, &store, context))
                            .collect();

                        let mut collected = Vec::new();
                        for res in &function_result {
                            if let Ok(v) = res {
                                collected.push(v.clone());
                            }
                        }

                        let list = Value {
                            kind: Some(Kind::ListValue(ListValue { values: collected })),
                        };

                        let is_faulty = function_result.iter().any(|res| res.is_err());

                        let entry = ContextEntry::new(
                            Result::Ok(list.clone()),
                            parameter_collection.clone(),
                        );

                        context.write_to_current_context(entry);

                        match !is_faulty {
                            true => {
                                // Add the value back to the main parameter
                                parameter_collection.push(list.clone());
                            }
                            false => {
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

fn handle_message(flow: ExecutionFlow, store: &FunctionStore) -> Option<Value> {
    let mut context = Context::new();

    if let Some(node) = flow.starting_node {
        match handle_node_function(node, store, &mut context) {
            Ok(result) => {
                println!(
                    "Execution completed successfully: The value is {:?}",
                    result
                );
                return Some(result);
            }
            Err(runtime_error) => {
                println!("Runtime Error: {:?}", runtime_error);
                return None;
            }
        }
    };
    return None;
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    load_env_file();

    let config = Config::new();
    let mut store = FunctionStore::new();
    store.populate(collect());

    let client = match async_nats::connect("nats://127.0.0.1:4222").await {
        Ok(client) => client,
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    };

    let _ = match client
        .queue_subscribe(String::from("execution.*"), "taurus".into())
        .await
    {
        Ok(mut sub) => {
            println!("Subscribed to 'execution.*'");

            while let Some(msg) = sub.next().await {
                let flow: ExecutionFlow = match ExecutionFlow::decode(&*msg.payload) {
                    Ok(flow) => flow,
                    Err(err) => {
                        println!("Failed to deserialize flow: {}, {:?}", err, &msg.payload);
                        continue;
                    }
                };

                let value = match handle_message(flow, &store) {
                    Some(value) => value,
                    None => Value {
                        kind: Some(Kind::NullValue(0)),
                    },
                };

                // Send a response to the reply subject
                if let Some(reply) = msg.reply {
                    match client.publish(reply, value.encode_to_vec().into()).await {
                        Ok(_) => println!("Response sent"),
                        Err(err) => println!("Failed to send response: {}", err),
                    }
                }
            }
        }
        Err(err) => panic!("Failed to subscribe to 'execution.*': {}", err),
    };
}
