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

    /// Verification Token required for internal communication
    pub rabbitmq_url: String,

    /// URL to the `Sagittarius` Server.
    pub aquila_url: String,
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
            rabbitmq_url: env_with_default("RABBITMQ_URL", String::from("amqp://localhost:5672")),
            aquila_url: env_with_default("AQUILA_URL", String::from("http://localhost:8080")),
        }
    }
}
