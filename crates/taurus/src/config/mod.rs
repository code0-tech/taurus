use code0_flow::flow_config::env_with_default;
use code0_flow::flow_config::environment::Environment;
use code0_flow::flow_config::mode::Mode;

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

    pub definitions: String,

    /// Runtime status heartbeat interval in seconds while Taurus is running.
    /// Set to 0 to disable periodic heartbeat updates.
    pub runtime_status_update_interval_seconds: u64,
}

/// Implementation for all relevant `Aquila` startup configurations
///
/// Behavior:
/// Searches for the env. file at root level. Filename: `.env`
impl Config {
    pub fn new() -> Self {
        Config {
            environment: env_with_default("ENVIRONMENT", Environment::Development),
            mode: env_with_default("MODE", Mode::DYNAMIC),
            nats_url: env_with_default("NATS_URL", String::from("nats://localhost:4222")),
            aquila_url: env_with_default("AQUILA_URL", String::from("http://localhost:50051")),
            aquila_token: env_with_default("AQUILA_TOKEN", String::from("token")),
            with_health_service: env_with_default("WITH_HEALTH_SERVICE", false),
            grpc_host: env_with_default("GRPC_HOST", "127.0.0.1".to_string()),
            grpc_port: env_with_default("GRPC_PORT", 50051),
            definitions: env_with_default("DEFINITIONS", String::from("./definitions")),
            runtime_status_update_interval_seconds: env_with_default(
                "RUNTIME_STATUS_UPDATE_INTERVAL_SECONDS",
                30_u64,
            ),
        }
    }
}
