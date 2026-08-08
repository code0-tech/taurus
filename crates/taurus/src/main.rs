//! Taurus is the CodeZero execution runtime: it consumes flow-execution
//! requests from NATS, runs them through `taurus_core::runtime::engine`, and
//! reports results back to Aquila over gRPC in dynamic mode.
//!
//! See [`app`] for the startup/shutdown sequence and the NATS worker loop,
//! [`client`] for the gRPC clients that talk back to Aquila, and [`config`]
//! for the environment-driven `Config` every other module reads from.

mod app;
mod client;
mod config;
mod telemetry;

fn main() {
    // .env must be loaded, and MAX_BLOCKING_THREADS read, before the Tokio
    // runtime is built below -- Tokio only accepts this setting at
    // construction time, so it can't live in the `Config` that app::run()
    // loads afterward.
    code0_flow::flow_config::load_env_file();
    let max_blocking_threads: usize =
        code0_flow::flow_config::env_with_default("MAX_BLOCKING_THREADS", 512_usize);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(max_blocking_threads)
        .build()
        .unwrap_or_else(|error| panic!("failed to build Tokio runtime: {error}"));

    runtime.block_on(app::run());
}
