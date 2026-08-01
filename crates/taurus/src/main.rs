//! Taurus is the CodeZero execution runtime: it consumes flow-execution
//! requests from NATS, runs them through `taurus_core::runtime::engine`, and
//! reports results back to Aquila over gRPC (dynamic mode) or purely via the
//! emitter (static mode).
//!
//! See [`app`] for the startup/shutdown sequence and the NATS worker loop,
//! [`client`] for the gRPC clients that talk back to Aquila, and [`config`]
//! for the environment-driven `Config` every other module reads from.

mod app;
mod client;
mod config;
mod handler_overrides;
mod telemetry;

#[tokio::main]
async fn main() {
    app::run().await;
}
