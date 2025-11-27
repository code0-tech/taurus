mod config;
pub mod context;
pub mod error;
pub mod implementation;

use crate::config::Config;
use crate::context::executor::Executor;
use crate::context::registry::FunctionStore;
use crate::context::signal::Signal;
use crate::implementation::collect;
use code0_flow::flow_service::FlowUpdateService;

use code0_flow::flow_config::load_env_file;
use code0_flow::flow_config::mode::Mode::DYNAMIC;
use context::context::Context;
use futures_lite::StreamExt;
use log::error;
use prost::Message;
use std::collections::HashMap;
use tokio::signal;
use tonic_health::pb::health_server::HealthServer;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, NodeFunction, Value};

fn handle_message(flow: ExecutionFlow, store: &FunctionStore) -> Signal {
    let context = Context::default();

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
        Ok(client) => {
            log::info!("Connected to nats server");
            client
        }
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    };

    // Optional health service task
    let health_task = if config.with_health_service {
        let health_service = code0_flow::flow_health::HealthService::new(config.nats_url.clone());
        let address = match format!("{}:{}", config.grpc_host, config.grpc_port).parse() {
            Ok(address) => address,
            Err(err) => {
                error!("Failed to parse grpc address: {:?}", err);
                return;
            }
        };

        log::info!("Health server starting at {}", address);

        Some(tokio::spawn(async move {
            if let Err(err) = tonic::transport::Server::builder()
                .add_service(HealthServer::new(health_service))
                .serve(address)
                .await
            {
                log::error!("Health server error: {:?}", err);
            } else {
                log::info!("Health server stopped gracefully.");
            }
        }))
    } else {
        None
    };

    // Optional: dynamic mode sync at startup
    if config.mode == DYNAMIC {
        FlowUpdateService::from_url(
            config.aquila_url.clone(),
            config.definitions.clone().as_str(),
        )
        .send()
        .await;
    }

    let mut worker_task = tokio::spawn(async move {
        let mut sub = match client
            .queue_subscribe(String::from("execution.*"), "taurus".into())
            .await
        {
            Ok(sub) => {
                log::info!("Subscribed to 'execution.*'");
                sub
            }
            Err(err) => {
                log::error!("Failed to subscribe to 'execution.*': {:?}", err);
                return;
            }
        };

        while let Some(msg) = sub.next().await {
            let flow: ExecutionFlow = match ExecutionFlow::decode(&*msg.payload) {
                Ok(flow) => flow,
                Err(err) => {
                    log::error!(
                        "Failed to deserialize flow: {:?}, payload: {:?}",
                        err,
                        &msg.payload
                    );
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
                    Ok(_) => log::info!("Response sent"),
                    Err(err) => log::error!("Failed to send response: {:?}", err),
                }
            }
        }

        log::info!("NATS worker loop ended");
    });

    match health_task {
        Some(mut health_task) => {
            // both are mutable JoinHandle<()> so we can borrow them in select!
            tokio::select! {
                _ = &mut worker_task => {
                    log::warn!("NATS worker task finished, shutting down");
                    health_task.abort();
                }
                _ = &mut health_task => {
                    log::warn!("Health server task finished, shutting down");
                    worker_task.abort();
                }
                _ = signal::ctrl_c() => {
                    log::info!("Ctrl+C/Exit signal received, shutting down");
                    worker_task.abort();
                    health_task.abort();
                }
            }
        }
        None => {
            tokio::select! {
                _ = &mut worker_task => {
                    log::warn!("NATS worker task finished, shutting down");
                }
                _ = signal::ctrl_c() => {
                    log::info!("Ctrl+C/Exit signal received, shutting down");
                    worker_task.abort();
                }
            }
        }
    }

    log::info!("Taurus shutdown complete");
}
