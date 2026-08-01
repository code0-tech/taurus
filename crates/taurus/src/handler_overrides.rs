//! Handler overrides applied on top of taurus-core's default registry.
//!
//! taurus-core's `http::request::send` is a fully synchronous, blocking
//! call (correct for non-Tokio callers like `taurus-tests`/`taurus-manual
//! --offline`), but this service runs many flows concurrently on a shared
//! Tokio worker pool, so a blocking HTTP call would stall unrelated
//! in-flight flows for its duration. `block_in_place` moves other queued
//! tasks off this worker thread while the blocking call runs, without
//! needing the handler calling convention itself to become async.

use taurus_core::handler::argument::Argument;
use taurus_core::handler::registry::ThunkRunner;
use taurus_core::runtime::engine::FunctionRegistration;
use taurus_core::runtime::execution::value_store::ValueStore;
use taurus_core::runtime::functions::http;
use taurus_core::types::signal::Signal;

pub fn all() -> Vec<FunctionRegistration> {
    vec![FunctionRegistration::eager(
        "http::request::send",
        http_request_non_blocking,
        8,
    )]
}

fn http_request_non_blocking(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut ThunkRunner<'_>,
) -> Signal {
    tokio::task::block_in_place(|| http::send_request(args, ctx, run))
}
