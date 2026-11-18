//! Process-wide [`Config`], loaded once from environment variables (with an
//! optional `.env` file) at startup and read by every other module that
//! needs a URL, timeout, or mode flag.

use code0_flow::flow_config::env_with_default;
use code0_flow::flow_config::environment::Environment;
use code0_flow::flow_config::mode::Mode;

use crate::telemetry::OpenTelemetry;

/// Struct for all relevant `Taurus` startup configurations
pub struct Config {
    pub environment: Environment,
    /// Taurus mode
    ///
    /// Options:
    /// `static` (default)
    /// `dynamic`
    pub mode: Mode,

    /// URL to the NATS service
    pub nats_url: String,

    pub aquila_url: String,

    pub aquila_token: String,

    pub with_health_service: bool,

    pub grpc_host: String,

    pub grpc_port: u16,

    /// Runtime status heartbeat interval in seconds while Taurus is running.
    /// Set to 0 to disable periodic heartbeat updates.
    pub runtime_status_update_interval_seconds: u64,

    /// Timeout in seconds for establishing Aquila gRPC channels.
    pub aquila_grpc_connect_timeout_secs: u64,

    /// Timeout in seconds for Aquila gRPC requests.
    pub aquila_grpc_request_timeout_secs: u64,

    /// Timeout in seconds for remote runtime NATS flush and response waits.
    pub remote_runtime_timeout_secs: u64,

    /// Maximum number of flow executions the worker runs concurrently.
    /// Bounds memory/connection use under a burst of NATS messages; the
    /// worker keeps pulling from the subscription but stops spawning new
    /// executions once this many are in flight. Defaults to 4x the
    /// available CPU parallelism (flows spend much of their time waiting
    /// on remote calls, not on CPU, so oversubscribing cores is reasonable
    /// as a starting point -- tune via env for your actual workload).
    pub max_concurrent_executions: usize,

    /// Number of distinct compiled flows kept warm in the in-process LRU
    /// cache, avoiding a recompile on every execution of the same flow.
    /// Set to 0 to disable the cache and recompile every execution.
    pub compiled_flow_cache_capacity: usize,

    /// Probability (0.0..=1.0) that a flow execution's full result --
    /// input and per-node results -- is reported to Aquila and forwarded to
    /// Sagittarius. Every execution still reports its outcome to Aquila
    /// (actions and test executions wait on it); executions Aquila marks as
    /// sampled (Sagittarius test executions) always report in full.
    pub execution_result_sample_rate: f64,

    /// JetStream work-queue stream (and subject) every execution's runtime
    /// usage is group-committed to; Aquila drains it and forwards the
    /// usages to Sagittarius.
    pub runtime_usage_stream: String,

    /// Maximum number of runtime usages published as one NATS message.
    pub runtime_usage_batch_size: usize,

    /// Maximum time in milliseconds a recorded runtime usage waits before
    /// its batch is published.
    pub runtime_usage_flush_interval_ms: u64,

    /// Maximum number of runtime usages buffered in memory while waiting to
    /// be published; further usages are dropped while the buffer is full.
    pub runtime_usage_buffer_capacity: usize,

    /// Estimated total bytes the compiled-flow cache may hold before
    /// evicting the least-recently-used entry, independent of
    /// `compiled_flow_cache_capacity` -- whichever limit is hit first wins.
    /// Bounds worst-case memory when individual flows are large (nothing
    /// in this service caps flow/message size upstream). Set to 0 to
    /// disable the cache and recompile every execution.
    pub compiled_flow_cache_max_bytes: usize,

    /// OpenTelemetry exporter configuration.
    pub opentelemetry: OpenTelemetry,

    /// Apus: accept direct action connections (`APUS_ENABLED`).
    pub apus_enabled: bool,
    pub apus_host: String,
    pub apus_port: u16,
    /// `EXEC` + `SUBFLOW` requests accepted concurrently per action connection.
    pub apus_max_in_flight: u32,
    pub apus_heartbeat_ms: u64,
    /// Attach a trace context to every `CALL`.
    pub apus_trace: bool,
    /// This instance in the routing table, and the endpoint actions reach it at.
    pub apus_instance_id: String,
    pub apus_advertise: String,
    /// Routing table file (`{"version", "instances": [{"id", "endpoint", "projects"}]}`),
    /// reloaded on change. Empty: this instance serves every project.
    pub apus_routing_path: String,
    /// TLS (all three or none): certificate chain, key and the client CA (Aquila's) whose
    /// certificates identify actions. Without TLS nobody is authenticated (development).
    pub apus_tls_cert: String,
    pub apus_tls_key: String,
    pub apus_tls_client_ca: String,
    /// Strings of at least this many bytes are sent as streams (0 = never).
    pub apus_stream_threshold: u32,
    pub apus_max_frame: u32,
    pub apus_max_connections: usize,
    pub apus_max_connections_per_action: usize,
    /// `nats` (default): queue action-triggered executions on NATS.
    /// `local`: execute them in this process.
    pub apus_execution: String,
    pub apus_flow_bucket: String,
    /// Cache flows looked up for `EXEC` instead of reading the KV store each time.
    pub apus_flow_cache: bool,
    /// Load flows into memory from this JSON export (`tucana::shared::Flows`) instead of
    /// the NATS KV store. Stands in for Aquila pushing flows to Taurus (control plane).
    pub apus_flows_path: String,
    /// `false`: no NATS at all (no connection, no `execution.*` worker, no KV store).
    pub nats_enabled: bool,
}

/// Implementation for all relevant `Taurus` startup configurations
///
/// Behavior:
/// Searches for the env. file at root level. Filename: `.env`
impl Config {
    pub fn new() -> Self {
        let max_concurrent_executions: usize = env_with_default(
            "MAX_CONCURRENT_EXECUTIONS",
            default_max_concurrent_executions(),
        );

        Config {
            environment: env_with_default("ENVIRONMENT", Environment::Development),
            mode: env_with_default("MODE", Mode::DYNAMIC),
            nats_url: env_with_default("NATS_URL", String::from("nats://localhost:4222")),
            aquila_url: env_with_default("AQUILA_URL", String::from("http://localhost:50051")),
            aquila_token: env_with_default("AQUILA_TOKEN", String::from("token")),
            with_health_service: env_with_default("WITH_HEALTH_SERVICE", false),
            grpc_host: env_with_default("GRPC_HOST", "127.0.0.1".to_string()),
            grpc_port: env_with_default("GRPC_PORT", 50051),
            runtime_status_update_interval_seconds: env_with_default(
                "RUNTIME_STATUS_UPDATE_INTERVAL_SECONDS",
                30_u64,
            ),
            aquila_grpc_connect_timeout_secs: env_with_default(
                "AQUILA_GRPC_CONNECT_TIMEOUT_SECS",
                2_u64,
            ),
            aquila_grpc_request_timeout_secs: env_with_default(
                "AQUILA_GRPC_REQUEST_TIMEOUT_SECS",
                10_u64,
            ),
            remote_runtime_timeout_secs: env_with_default("REMOTE_RUNTIME_TIMEOUT_SECS", 30_u64),
            max_concurrent_executions,
            compiled_flow_cache_capacity: env_with_default(
                "COMPILED_FLOW_CACHE_CAPACITY",
                taurus_core::runtime::engine::DEFAULT_COMPILED_FLOW_CACHE_CAPACITY,
            ),
            compiled_flow_cache_max_bytes: env_with_default(
                "COMPILED_FLOW_CACHE_MAX_BYTES",
                taurus_core::runtime::engine::DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES,
            ),
            execution_result_sample_rate: sample_rate(env_with_default(
                "EXECUTION_RESULT_SAMPLE_RATE",
                1.0_f64,
            )),
            runtime_usage_stream: env_with_default(
                "RUNTIME_USAGE_STREAM",
                String::from("runtime_usage"),
            ),
            runtime_usage_batch_size: env_with_default("RUNTIME_USAGE_BATCH_SIZE", 1000_usize),
            runtime_usage_flush_interval_ms: env_with_default(
                "RUNTIME_USAGE_FLUSH_INTERVAL_MS",
                1000_u64,
            ),
            runtime_usage_buffer_capacity: env_with_default(
                "RUNTIME_USAGE_BUFFER_CAPACITY",
                100_000_usize,
            ),
            opentelemetry: OpenTelemetry {
                enabled: env_with_default("OPENTELEMETRY_ENABLED", false),
                service_name: env_with_default(
                    "OPENTELEMETRY_SERVICE_NAME",
                    env!("CARGO_PKG_NAME").to_string(),
                ),
                logs_endpoint: optional_env("OPENTELEMETRY_LOGS_ENDPOINT"),
                metrics_endpoint: optional_env("OPENTELEMETRY_METRICS_ENDPOINT"),
                traces_endpoint: optional_env("OPENTELEMETRY_TRACES_ENDPOINT"),
            },
            apus_enabled: env_with_default("APUS_ENABLED", false),
            apus_host: env_with_default("APUS_HOST", String::from("127.0.0.1")),
            apus_port: env_with_default("APUS_PORT", 7676_u16),
            apus_max_in_flight: env_with_default("APUS_MAX_IN_FLIGHT", 4096_u32),
            apus_heartbeat_ms: env_with_default("APUS_HEARTBEAT_MS", 10_000_u64),
            apus_trace: env_with_default("APUS_TRACE", false),
            apus_instance_id: env_with_default("APUS_INSTANCE_ID", String::from("taurus-local")),
            apus_advertise: env_with_default("APUS_ADVERTISE", String::new()),
            apus_routing_path: env_with_default("APUS_ROUTING_PATH", String::new()),
            apus_tls_cert: env_with_default("APUS_TLS_CERT", String::new()),
            apus_tls_key: env_with_default("APUS_TLS_KEY", String::new()),
            apus_tls_client_ca: env_with_default("APUS_TLS_CLIENT_CA", String::new()),
            apus_stream_threshold: env_with_default("APUS_STREAM_THRESHOLD", 64 * 1024_u32),
            apus_max_frame: env_with_default("APUS_MAX_FRAME", 16 * 1024 * 1024_u32),
            apus_max_connections: env_with_default("APUS_MAX_CONNECTIONS", 10_000_usize),
            apus_max_connections_per_action: env_with_default(
                "APUS_MAX_CONNECTIONS_PER_ACTION",
                64_usize,
            ),
            apus_execution: env_with_default("APUS_EXECUTION", String::from("nats")),
            apus_flow_bucket: env_with_default("APUS_FLOW_BUCKET", String::from("flow_store")),
            apus_flow_cache: env_with_default("APUS_FLOW_CACHE", false),
            apus_flows_path: env_with_default("APUS_FLOWS_PATH", String::new()),
            nats_enabled: env_with_default("NATS_ENABLED", true),
        }
    }
}

fn default_max_concurrent_executions() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .saturating_mul(4)
}

/// Clamps a configured sample rate into `0.0..=1.0`; anything that isn't a
/// number falls back to reporting every result.
fn sample_rate(rate: f64) -> f64 {
    if rate.is_nan() {
        1.0
    } else {
        rate.clamp(0.0, 1.0)
    }
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
