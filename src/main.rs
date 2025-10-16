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
use log::error;
use prost::Message;
use registry::FunctionStore;
use std::collections::HashMap;
use tonic_health::pb::health_server::HealthServer;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, ListValue, NodeFunction, Value};

fn handle_node_function(
    function: NodeFunction,
    node_functions: &HashMap<i64, NodeFunction>,
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
                    tucana::shared::node_value::Value::NodeFunctionId(another_node_function) => {
                        // As this is another new indent, a new context will be opened

                        let function_result = match node_functions.get(&another_node_function) {
                            Some(function_result) => {
                                context.next_context();
                                handle_node_function(function_result.clone(), node_functions, store, context )
                            },
                            None => {
                                todo!("Handle node not found. This should normally not happen")
                            }
                        };


                        let entry = ContextEntry::new(
                            Result::Ok(function_result.clone()?),
                            parameter_collection.clone(),
                        );

                        context.write_to_current_context(entry);
                    }
                }
            }
        }

        let result = runtime_function(&parameter_collection, context);

        // Result will be added to the current context
        let entry = ContextEntry::new(result.clone(), parameter_collection.clone());
        context.write_to_current_context(entry);

        // Check if there is a next node, if not then this was the last one
        match function.next_node_id {
            Some(next_node_function_id) => {
                let node = match node_functions.get(&next_node_function_id) {
                    Some(node) => node,
                    None => todo!("Handle node not found. This should normally not happen"),
                };

                // Increment the context node!
                context.next_node();

                return handle_node_function(node.clone(), node_functions, store, context);
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

    let node_functions: HashMap<i64, NodeFunction> = flow
        .node_functions
        .into_iter()
        .map(|node| return (node.database_id, node))
        .collect();

    if let Some(node) = node_functions.get(&flow.starting_node_id) {
        return match handle_node_function(node.clone(), &node_functions, store, &mut context) {
            Ok(result) => {
                println!(
                    "Execution completed successfully: The value is {:?}",
                    result
                );
                Some(result)
            }
            Err(runtime_error) => {
                println!("Runtime Error: {:?}", runtime_error);
                None
            }
        };
    }
    None
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

    let client = match async_nats::connect(config.nats_url.clone()).await {
        Ok(client) => client,
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    };

    if config.with_health_service {
        let health_service = code0_flow::flow_health::HealthService::new(config.nats_url.clone());
        let address = match format!("{}:{}", config.grpc_host, config.grpc_port).parse() {
            Ok(address) => address,
            Err(err) => {
                error!("Failed to parse grpc address: {:?}", err);
                return;
            }
        };

        tokio::spawn(async move {
            let _ = tonic::transport::Server::builder()
                .add_service(HealthServer::new(health_service))
                .serve(address)
                .await;
        });

        println!("Health server started at {}", address);
    }

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

                let value = handle_message(flow, &store).unwrap_or_else(|| Value {
                    kind: Some(Kind::NullValue(0)),
                });

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
