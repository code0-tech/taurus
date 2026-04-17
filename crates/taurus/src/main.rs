mod client;
mod config;
mod remote;

use crate::client::runtime_status::TaurusRuntimeStatusService;
use crate::client::runtime_usage::TaurusRuntimeUsageService;
use crate::config::Config;
use crate::remote::RemoteNatsClient;
use code0_flow::flow_service::FlowUpdateService;

use code0_flow::flow_config::load_env_file;
use code0_flow::flow_config::mode::Mode::DYNAMIC;
use futures_lite::StreamExt;
use log::error;
use prost::Message;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use taurus_core::context::context::Context;
use taurus_core::context::executor::Executor;
use taurus_core::context::registry::FunctionStore;
use taurus_core::context::signal::Signal;
use taurus_core::runtime::error::RuntimeError;
use tokio::signal;
use tokio::time::sleep;
use tonic_health::pb::health_server::HealthServer;
use tucana::shared::value::Kind;
use tucana::shared::{
    ExecutionFlow, NodeFunction, RuntimeFeature, RuntimeUsage, Translation, Value,
};

fn handle_message(
    flow: ExecutionFlow,
    store: &FunctionStore,
    nats_remote: &RemoteNatsClient,
) -> (Signal, RuntimeUsage) {
    let start = Instant::now();
    let mut context = match flow.input_value {
        Some(v) => {
            log::debug!("Input Value for flow: {:?}", v);
            Context::new(v)
        }
        None => Context::default(),
    };

    if flow.node_functions.len() == 0 {
        let duration_millis = start.elapsed().as_millis() as i64;
        return (
            Signal::Failure(RuntimeError::simple_str(
                "InvalidFlow",
                "This flow has no nodes to execute!",
            )),
            RuntimeUsage {
                flow_id: flow.flow_id,
                duration: duration_millis,
            },
        );
    }

    let node_functions: HashMap<i64, NodeFunction> = flow
        .node_functions
        .into_iter()
        .map(|node| (node.database_id, node))
        .collect();

    let signal = Executor::new(store, node_functions)
        .with_remote_runtime(nats_remote)
        .execute(flow.starting_node_id, &mut context, true);
    let duration_millis = start.elapsed().as_millis() as i64;

    (
        signal,
        RuntimeUsage {
            flow_id: flow.flow_id,
            duration: duration_millis,
        },
    )
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    load_env_file();

    let config = Config::new();
    let store = FunctionStore::default();
    let mut runtime_status_service: Option<TaurusRuntimeStatusService> = None;
    let mut runtime_usage_service: Option<TaurusRuntimeUsageService> = None;

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

    if config.mode == DYNAMIC {
        let definition_service = FlowUpdateService::from_url(
            config.aquila_url.clone(),
            config.definitions.clone().as_str(),
            config.aquila_token.clone(),
        )
        .await;

        let mut success = false;
        let mut count = 1;
        while !success {
            success = definition_service.send_with_status().await;
            if success {
                break;
            }

            log::warn!(
                "Updating definitions failed, trying again in 2 secs (retry number {})",
                count
            );
            count += 1;
            sleep(Duration::from_secs(3)).await;
        }

        let usage_service = TaurusRuntimeUsageService::from_url(
            config.aquila_url.clone(),
            config.aquila_token.clone(),
        )
        .await;
        runtime_usage_service = Some(usage_service);

        let status_service = TaurusRuntimeStatusService::from_url(
            config.aquila_url.clone(),
            config.aquila_token.clone(),
            "taurus".into(),
            vec![RuntimeFeature {
                name: vec![Translation {
                    code: "en-US".to_string(),
                    content: "Runtime".to_string(),
                }],
                description: vec![Translation {
                    code: "en-US".to_string(),
                    content: "Will execute incoming flows.".to_string(),
                }],
            }],
        )
        .await;

        status_service
            .update_runtime_status(tucana::shared::execution_runtime_status::Status::Running)
            .await;
        runtime_status_service = Some(status_service);
    }

    let nats_client = RemoteNatsClient::new(client.clone());
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

            let flow_id = flow.flow_id;
            let result = handle_message(flow, &store, &nats_client);
            let value = match result.0 {
                Signal::Failure(error) => {
                    log::error!(
                        "RuntimeError occurred, execution failed because: {:?}",
                        error
                    );
                    error.as_value()
                }
                Signal::Success(v) => {
                    log::debug!("Execution ended on a success signal");
                    v
                }
                Signal::Return(v) => {
                    log::debug!("Execution ended on a return signal");
                    v
                }
                Signal::Respond(v) => {
                    log::debug!("Execution ended on a respond signal");
                    v
                }
                Signal::Stop => {
                    log::debug!("Revied stop signal as last signal");
                    Value {
                        kind: Some(Kind::NullValue(0)),
                    }
                }
            };

            log::info!("For the flow_id {} returing the value {:?}", flow_id, value);

            // Send a response to the reply subject
            if let Some(reply) = msg.reply {
                match client.publish(reply, value.encode_to_vec().into()).await {
                    Ok(_) => log::debug!("Response sent"),
                    Err(err) => log::error!("Failed to send response: {:?}", err),
                }
            }

            if let Some(usage_service) = &runtime_usage_service {
                usage_service.update_runtime_usage(result.1).await;
            }
        }

        log::info!("NATS worker loop ended");
    });

    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    match health_task {
        Some(mut health_task) => {
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
                _ = sigterm => {
                    log::info!("SIGTERM received, shutting down");
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
                _ = sigterm => {
                    log::info!("SIGTERM received, shutting down");
                    worker_task.abort();
                }
            }
        }
    }

    if let Some(status_service) = &runtime_status_service {
        status_service
            .update_runtime_status(tucana::shared::execution_runtime_status::Status::Stopped)
            .await;
    };

    log::info!("Taurus shutdown complete");
}
