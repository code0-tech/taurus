mod worker;

use code0_flow::flow_config::load_env_file;
use code0_flow::flow_config::mode::Mode::DYNAMIC;
use code0_flow::flow_service::FlowUpdateService;
use std::time::Duration;
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_provider::providers::emitter::nats_emitter::NATSRespondEmitter;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::signal;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic_health::pb::health_server::HealthServer;

use crate::client::runtime_status::TaurusRuntimeStatusService;
use crate::client::runtime_usage::TaurusRuntimeUsageService;
use crate::config::Config;

pub async fn run() {
    init_logging();
    load_env_file();

    let config = Config::new();
    let engine = ExecutionEngine::new();
    let client = connect_nats(&config).await;

    let mut health_task = spawn_health_task(&config);
    let (runtime_status_service, runtime_usage_service) =
        setup_dynamic_services_if_needed(&config).await;

    let nats_remote = NATSRemoteRuntime::new(client.clone());
    let runtime_emitter = NATSRespondEmitter::new(client.clone());
    let mut worker_task = worker::spawn_worker(
        client,
        engine,
        nats_remote,
        runtime_emitter,
        runtime_usage_service,
    );

    wait_for_shutdown(&mut worker_task, &mut health_task).await;
    update_stopped_status(runtime_status_service.as_ref()).await;

    log::info!("Taurus shutdown complete");
}

fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

async fn connect_nats(config: &Config) -> async_nats::Client {
    match async_nats::connect(config.nats_url.clone()).await {
        Ok(client) => {
            log::info!("Connected to NATS server");
            client
        }
        Err(err) => {
            panic!("Failed to connect to NATS server: {}", err);
        }
    }
}

fn spawn_health_task(config: &Config) -> Option<JoinHandle<()>> {
    if !config.with_health_service {
        return None;
    }

    let health_service = code0_flow::flow_health::HealthService::new(config.nats_url.clone());
    let address = match format!("{}:{}", config.grpc_host, config.grpc_port).parse() {
        Ok(address) => address,
        Err(err) => {
            log::error!("Failed to parse gRPC address: {:?}", err);
            return None;
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
            log::info!("Health server stopped gracefully");
        }
    }))
}

async fn setup_dynamic_services_if_needed(
    config: &Config,
) -> (
    Option<TaurusRuntimeStatusService>,
    Option<TaurusRuntimeUsageService>,
) {
    if config.mode != DYNAMIC {
        return (None, None);
    }

    push_definitions_until_success(config).await;

    let runtime_usage_service = Some(
        TaurusRuntimeUsageService::from_url(config.aquila_url.clone(), config.aquila_token.clone())
            .await,
    );

    let runtime_status_service = Some(
        TaurusRuntimeStatusService::from_url(
            config.aquila_url.clone(),
            config.aquila_token.clone(),
            "taurus".into(),
        )
        .await,
    );

    if let Some(status_service) = runtime_status_service.as_ref() {
        status_service
            .update_runtime_status(tucana::shared::execution_runtime_status::Status::Running)
            .await;
    }

    (runtime_status_service, runtime_usage_service)
}

async fn push_definitions_until_success(config: &Config) {
    let mut definition_service = FlowUpdateService::from_url(
        config.aquila_url.clone(),
        config.definitions.as_str(),
        config.aquila_token.clone(),
    )
    .await
    .with_definition_source(String::from("taurus"));

    let mut retry_count = 1;
    loop {
        if definition_service.send_with_status().await {
            break;
        }

        log::warn!(
            "Updating definitions failed, trying again in 3 seconds (retry #{})",
            retry_count
        );
        retry_count += 1;
        sleep(Duration::from_secs(3)).await;
    }
}

async fn update_stopped_status(runtime_status_service: Option<&TaurusRuntimeStatusService>) {
    if let Some(status_service) = runtime_status_service {
        status_service
            .update_runtime_status(tucana::shared::execution_runtime_status::Status::Stopped)
            .await;
    }
}

async fn wait_for_shutdown(
    worker_task: &mut JoinHandle<()>,
    health_task: &mut Option<JoinHandle<()>>,
) {
    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    if let Some(health_task) = health_task.as_mut() {
        tokio::select! {
            _ = &mut *worker_task => {
                log::warn!("NATS worker task finished, shutting down");
                health_task.abort();
            }
            _ = &mut *health_task => {
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
    } else {
        tokio::select! {
            _ = &mut *worker_task => {
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
