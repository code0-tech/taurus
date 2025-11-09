mod config;
pub mod context;
pub mod error;
pub mod implementation;

use crate::config::Config;
use crate::context::signal::Signal;
use crate::implementation::collect;
use code0_flow::flow_config::load_env_file;
use context::Context;
use futures_lite::StreamExt;
use log::error;
use prost::Message;
use std::collections::HashMap;
use tonic_health::pb::health_server::HealthServer;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, NodeFunction, Value};
use crate::context::executor::Executor;
use crate::context::registry::FunctionStore;

fn handle_message(flow: ExecutionFlow, store: &FunctionStore) -> Signal {
    let context = Context::new();

    let node_functions: HashMap<i64, NodeFunction> = flow
        .node_functions
        .into_iter()
        .map(|node| (node.database_id, node))
        .collect();

    Executor::new(store, node_functions, context).execute(flow.starting_node_id)
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

    match client
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
                    Signal::Failure(error) => error.as_value(),
                    Signal::Success(v) => v,
                    Signal::Return(v) => v,
                    Signal::Respond(v) => v,
                    Signal::Stop => Value {
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
