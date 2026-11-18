//! Startup, shutdown, and (in dynamic mode) Aquila registration for the
//! Taurus service. [`worker`] is the actual NATS-driven flow execution loop
//! this module spawns and supervises; everything else here is one-time setup
//! or the shutdown sequence that tears it back down.
//!
//! Shutdown is cooperative rather than an abort: [`wait_for_shutdown`] signals
//! the worker task to stop via a `Notify` and waits for it to exit on its own,
//! so in-flight flow executions can finish before the process exits.

pub mod apus;
mod definitions;
mod worker;

use code0_flow::flow_config::environment::Environment;
use code0_flow::flow_config::mode::Mode::{DYNAMIC, STATIC};
use std::sync::Arc;
use std::time::Duration;
use taurus_core::runtime::engine::ExecutionEngine;
use taurus_provider::providers::remote::nats_remote_runtime::NATSRemoteRuntime;
use tokio::signal;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tonic_health::pb::health_server::HealthServer;
use tucana::shared::module_status::StatusVariant;

use crate::client::runtime_execution::TaurusRuntimeExecutionService;
use crate::client::runtime_status::TaurusRuntimeStatusService;
use crate::client::runtime_usage::{self, RuntimeUsageRecorder, RuntimeUsageSettings};
use crate::config::Config;
use crate::telemetry::{self, TelemetrySettings, errors};

pub async fn run() {
    // .env is loaded in main(), before the Tokio runtime is built.
    let config = Config::new();
    let telemetry = init_telemetry(&config);
    install_panic_logging();
    let engine = ExecutionEngine::with_compiled_flow_cache_limits(
        config.compiled_flow_cache_capacity,
        config.compiled_flow_cache_max_bytes,
    );
    let client = if config.nats_enabled {
        Some(connect_nats(&config).await)
    } else {
        log::info!("NATS disabled: no execution queue, Apus only");
        None
    };

    let mut health_task = spawn_health_task(&config);
    let (runtime_status_service, runtime_execution_service, mut runtime_status_heartbeat_task) =
        setup_dynamic_services_if_needed(&config).await;
    let (runtime_usage, runtime_usage_task) =
        spawn_runtime_usage_if_needed(&config, client.as_ref());

    let nats_remote = client.as_ref().map(|client| {
        NATSRemoteRuntime::with_execution_result_timeout(
            client.clone(),
            Duration::from_secs(config.remote_runtime_timeout_secs),
        )
    });
    let engine = Arc::new(engine);
    let remote_timeout = Duration::from_secs(config.remote_runtime_timeout_secs);
    let apus_hub = config.apus_enabled.then(|| {
        Arc::new(apus::ApusHub::new(
            &config.apus_instance_id,
            &apus_advertise(&config),
        ))
    });
    let remote = apus::RoutingRemote::new(
        apus_hub.clone(),
        nats_remote,
        remote_timeout,
        config.apus_trace,
    );
    let mut apus_exec = None;
    if let Some(hub) = &apus_hub {
        apus_exec = Some(
            start_apus_server(
                &config,
                hub.clone(),
                client.as_ref(),
                engine.clone(),
                remote.clone(),
                runtime_usage.clone(),
            )
            .await,
        );
    }
    let worker_shutdown = Arc::new(Notify::new());
    let mut worker_task = match client.clone() {
        Some(client) => worker::spawn_worker(
            client,
            engine,
            remote,
            worker::ExecutionReporting {
                runtime_execution_service,
                runtime_usage,
                result_sample_rate: config.execution_result_sample_rate,
            },
            mode_label(&config).to_string(),
            worker_shutdown.clone(),
            config.max_concurrent_executions,
            config.environment == Environment::Development,
        ),
        // No queue worker without NATS; this task only ends on shutdown.
        None => {
            let shutdown = worker_shutdown.clone();
            tokio::spawn(async move { shutdown.notified().await })
        }
    };

    wait_for_shutdown(&mut worker_task, &mut health_task, &worker_shutdown).await;
    if let Some(hub) = &apus_hub {
        hub.goaway_all("taurus is shutting down");
    }
    if let Some(exec) = &apus_exec {
        exec.stop_runtime_usage();
    }
    // The worker owned every runtime usage recorder, so with it finished the
    // publisher flushes what's left and exits on its own.
    if let Some(handle) = runtime_usage_task
        && let Err(err) = handle.await
    {
        log::warn!("Runtime usage publisher ended unexpectedly: {}", err);
        errors::record("task", "runtime_usage.task", &err, "mode=dynamic");
    }
    if let Some(handle) = runtime_status_heartbeat_task.take() {
        handle.abort();
        if let Err(err) = handle.await
            && !err.is_cancelled()
        {
            log::warn!("Runtime status heartbeat task ended unexpectedly: {}", err);
            errors::record(
                "task",
                "runtime_status_heartbeat.task",
                &err,
                "mode=dynamic",
            );
        }
    }
    update_stopped_status(runtime_status_service.as_ref()).await;

    log::info!("Taurus shutdown complete");
    telemetry.shutdown();
}

/// Accepts direct action connections (Apus data plane) next to the NATS worker.
async fn start_apus_server(
    config: &Config,
    hub: Arc<apus::ApusHub>,
    client: Option<&async_nats::Client>,
    engine: Arc<ExecutionEngine>,
    remote: apus::RoutingRemote,
    runtime_usage: Option<RuntimeUsageRecorder>,
) -> Arc<apus::ExecContext> {
    let flows = if !config.apus_flows_path.is_empty() {
        let flows = apus::load_flows_file(&config.apus_flows_path);
        log::info!("Apus: {} flow(s) loaded into memory", flows.len());
        apus::FlowSource::Memory(flows)
    } else {
        let client = client.expect("Apus: NATS is disabled, set APUS_FLOWS_PATH");
        let jetstream = async_nats::jetstream::new(client.clone());
        let kv = match jetstream
            .get_key_value(config.apus_flow_bucket.clone())
            .await
        {
            Ok(kv) => kv,
            Err(err) => panic!(
                "Apus: cannot open flow KV bucket '{}': {}",
                config.apus_flow_bucket, err
            ),
        };
        apus::FlowSource::Kv {
            kv,
            cache: config
                .apus_flow_cache
                .then(|| Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()))),
        }
    };
    let mode = match (client, config.apus_execution.as_str()) {
        (None, _) | (_, "local") => apus::ExecutionMode::Local,
        _ => apus::ExecutionMode::Nats,
    };
    log::info!(
        "Apus enabled execution_mode={:?} nats={} flows_in_memory={} flow_cache={}",
        mode,
        client.is_some(),
        !config.apus_flows_path.is_empty(),
        config.apus_flow_cache
    );
    let exec = Arc::new(apus::ExecContext {
        client: client.cloned(),
        flows,
        mode,
        engine,
        remote,
        timeout: Duration::from_secs(config.remote_runtime_timeout_secs),
        flow_type: mode_label(config).to_string(),
        with_trace: config.environment == Environment::Development,
        runtime_usage: std::sync::Mutex::new(runtime_usage),
    });
    exec.watch_flows();
    // With TLS, connections are mutual: the client certificate (issued by Aquila's CA)
    // identifies the action. Without TLS (development) nobody is authenticated.
    let tls_files = [
        &config.apus_tls_cert,
        &config.apus_tls_key,
        &config.apus_tls_client_ca,
    ];
    let tls = if tls_files.iter().all(|path| path.is_empty()) {
        log::warn!("Apus without TLS: actions are not authenticated (development only)");
        None
    } else if tls_files.iter().any(|path| path.is_empty()) {
        panic!(
            "Apus: TLS needs APUS_TLS_CERT, APUS_TLS_KEY and APUS_TLS_CLIENT_CA (client certificates identify actions)"
        );
    } else {
        let read = |path: &str| {
            std::fs::read(path).unwrap_or_else(|err| panic!("Apus: cannot read {path}: {err}"))
        };
        let server = apus_protocol::tls::server_config(
            &read(&config.apus_tls_cert),
            &read(&config.apus_tls_key),
            Some(&read(&config.apus_tls_client_ca)),
        )
        .unwrap_or_else(|err| panic!("Apus: {err}"));
        Some(apus_protocol::tls::TlsAcceptor::from(server))
    };
    if !config.apus_routing_path.is_empty() {
        apus::watch_routes(
            hub.clone(),
            config.apus_routing_path.clone(),
            config.apus_instance_id.clone(),
        );
    }
    let settings = apus::ServerSettings {
        host: config.apus_host.clone(),
        port: config.apus_port,
        instance_id: config.apus_instance_id.clone(),
        max_in_flight: config.apus_max_in_flight,
        heartbeat: Duration::from_millis(config.apus_heartbeat_ms),
        max_frame: config.apus_max_frame,
        stream_window: apus_protocol::limits::DEFAULT_STREAM_WINDOW,
        stream_threshold: config.apus_stream_threshold,
        tls,
        max_connections: config.apus_max_connections,
        max_connections_per_action: config.apus_max_connections_per_action,
        handshake_timeout: Duration::from_millis(apus_protocol::limits::HANDSHAKE_TIMEOUT_MS),
    };
    tokio::spawn(apus::serve(settings, hub, exec.clone()));
    exec
}

/// The endpoint other parties reach this Apus server at (for `ROUTES`).
fn apus_advertise(config: &Config) -> String {
    if config.apus_advertise.is_empty() {
        format!("{}:{}", config.apus_host, config.apus_port)
    } else {
        config.apus_advertise.clone()
    }
}

fn init_telemetry(config: &Config) -> telemetry::Telemetry {
    telemetry::Telemetry::initialize(
        &config.opentelemetry,
        TelemetrySettings {
            environment: environment_label(&config.environment),
            default_log_level: "debug",
            service_version: taurus_core::version::runtime_version(),
            instrumentation_name: env!("CARGO_PKG_NAME"),
            initialize_metrics: Some(telemetry::metrics::initialize),
        },
    )
    .unwrap_or_else(|error| panic!("failed to initialize telemetry: {error}"))
}

fn install_panic_logging() {
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            *message
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            message.as_str()
        } else {
            "<non-string panic payload>"
        };

        let location = panic_info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown".into());
        errors::panic(message, &location);
    }));
}

fn environment_label(environment: &Environment) -> &'static str {
    match environment {
        Environment::Development => "development",
        Environment::Staging => "staging",
        Environment::Production => "production",
    }
}

fn mode_label(config: &Config) -> &'static str {
    match config.mode {
        STATIC => "static",
        DYNAMIC => "dynamic",
    }
}

async fn connect_nats(config: &Config) -> async_nats::Client {
    match async_nats::connect(config.nats_url.clone()).await {
        Ok(client) => {
            log::info!("Connected to NATS server");
            client
        }
        Err(err) => {
            errors::record("transport", "nats.connect", &err, "component=nats");
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
            errors::record(
                "configuration",
                "health.address.parse",
                &err,
                "service=health",
            );
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
            errors::record("server", "health.serve", &err, "service=health");
        } else {
            log::info!("Health server stopped gracefully");
        }
    }))
}

async fn setup_dynamic_services_if_needed(
    config: &Config,
) -> (
    Option<Arc<TaurusRuntimeStatusService>>,
    Option<TaurusRuntimeExecutionService>,
    Option<JoinHandle<()>>,
) {
    if config.mode != DYNAMIC {
        return (None, None, None);
    }

    definitions::push_definitions_until_success(config).await;

    let runtime_execution_service = Some(
        TaurusRuntimeExecutionService::from_url(
            config.aquila_url.clone(),
            config.aquila_token.clone(),
            Duration::from_secs(config.aquila_grpc_connect_timeout_secs),
            Duration::from_secs(config.aquila_grpc_request_timeout_secs),
        )
        .await,
    );

    let runtime_status_service = Some(Arc::new(
        TaurusRuntimeStatusService::from_url(
            config.aquila_url.clone(),
            config.aquila_token.clone(),
            definitions::module_identifiers(),
            Duration::from_secs(config.aquila_grpc_connect_timeout_secs),
            Duration::from_secs(config.aquila_grpc_request_timeout_secs),
        )
        .await,
    ));

    if let Some(status_service) = runtime_status_service.as_ref() {
        status_service
            .update_runtime_status(StatusVariant::Running)
            .await;
    }

    let runtime_status_heartbeat_task = if config.runtime_status_update_interval_seconds > 0 {
        let status_service = runtime_status_service
            .as_ref()
            .expect("runtime status service should exist in dynamic mode")
            .clone();
        let update_interval_seconds = config.runtime_status_update_interval_seconds;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(update_interval_seconds));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            // First tick is immediate; consume it so heartbeats start after the interval.
            interval.tick().await;

            loop {
                interval.tick().await;
                status_service
                    .update_runtime_status(StatusVariant::Running)
                    .await;
            }
        });

        log::info!(
            "Runtime status heartbeat started (interval={}s)",
            update_interval_seconds
        );
        Some(handle)
    } else {
        log::info!("Runtime status heartbeat is disabled");
        None
    };

    (
        runtime_status_service,
        runtime_execution_service,
        runtime_status_heartbeat_task,
    )
}

/// Starts the runtime usage group-commit publisher in dynamic mode, where
/// Aquila drains the usage stream and forwards it to Sagittarius.
fn spawn_runtime_usage_if_needed(
    config: &Config,
    client: Option<&async_nats::Client>,
) -> (Option<RuntimeUsageRecorder>, Option<JoinHandle<()>>) {
    let Some(client) = client.filter(|_| config.mode == DYNAMIC) else {
        return (None, None);
    };

    let (recorder, handle) = runtime_usage::spawn(
        client.clone(),
        RuntimeUsageSettings {
            stream: config.runtime_usage_stream.clone(),
            batch_size: config.runtime_usage_batch_size,
            flush_interval: Duration::from_millis(config.runtime_usage_flush_interval_ms),
            buffer_capacity: config.runtime_usage_buffer_capacity,
        },
    );
    (Some(recorder), Some(handle))
}

async fn update_stopped_status(runtime_status_service: Option<&Arc<TaurusRuntimeStatusService>>) {
    if let Some(status_service) = runtime_status_service {
        status_service
            .update_runtime_status(StatusVariant::Stopped)
            .await;
    }
}

async fn wait_for_shutdown(
    worker_task: &mut JoinHandle<()>,
    health_task: &mut Option<JoinHandle<()>>,
    worker_shutdown: &Arc<Notify>,
) {
    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    let mut worker_already_finished = false;

    if let Some(health_task) = health_task.as_mut() {
        tokio::select! {
            _ = &mut *worker_task => {
                log::warn!("NATS worker task finished, shutting down");
                worker_already_finished = true;
                health_task.abort();
            }
            _ = &mut *health_task => {
                log::warn!("Health server task finished, shutting down");
                worker_shutdown.notify_one();
            }
            _ = signal::ctrl_c() => {
                log::info!("Ctrl+C/Exit signal received, shutting down");
                worker_shutdown.notify_one();
                health_task.abort();
            }
            _ = sigterm => {
                log::info!("SIGTERM received, shutting down");
                worker_shutdown.notify_one();
                health_task.abort();
            }
        }
    } else {
        tokio::select! {
            _ = &mut *worker_task => {
                log::warn!("NATS worker task finished, shutting down");
                worker_already_finished = true;
            }
            _ = signal::ctrl_c() => {
                log::info!("Ctrl+C/Exit signal received, shutting down");
                worker_shutdown.notify_one();
            }
            _ = sigterm => {
                log::info!("SIGTERM received, shutting down");
                worker_shutdown.notify_one();
            }
        }
    }

    // Let every in-flight execution finish so authoritative results aren't
    // silently dropped during shutdown.
    if !worker_already_finished && let Err(err) = worker_task.await {
        log::warn!(
            "NATS worker task ended unexpectedly during shutdown: {}",
            err
        );
    }
}
