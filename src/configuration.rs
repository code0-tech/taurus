use code0_flow::flow_config::{env_with_default, environment::Environment, mode::Mode};

/// Struct for all relevant `Taurus` startup configurations
pub struct Config {
    /// Options:
    /// `development` (default)
    /// `staging`
    /// `production`
    pub environment: Environment,

    /// Aquila mode
    ///
    /// Options:
    /// `static` (default)
    /// `hybrid`
    pub mode: Mode,

    pub nats_url: String,

    pub aquila_url: String,

    pub with_health_service: bool,

    pub grpc_host: String,

    pub grpc_port: u16,
}

/// Implementation for all relevant `Aquila` startup configurations
///
/// Behavior:
/// Searches for the env. file at root level. Filename: `.env`
impl Config {
    pub fn new() -> Self {
        Config {
            environment: env_with_default("ENVIRONMENT", Environment::Development),
            mode: env_with_default("MODE", Mode::STATIC),
            nats_url: env_with_default("NATS_URL", String::from("nats://localhost:4222")),
            aquila_url: env_with_default("AQUILA_URL", String::from("http://localhost:50051")),
            with_health_service: env_with_default("WITH_HEALTH_SERVICE", false),
            grpc_host: env_with_default("GRPC_HOST", "127.0.0.1".to_string()),
            grpc_port: env_with_default("GRPC_PORT", 50051),
        }
    }
}
