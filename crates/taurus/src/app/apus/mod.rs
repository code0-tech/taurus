//! Apus v1: direct connections between Taurus and actions.
//! See `Code0/apus/protocol/SPEC-v1.md`.

pub mod convert;
pub mod exec;
pub mod hub;
pub mod remote;

pub use exec::{ExecContext, ExecutionMode, FlowSource, load_flows_file};
pub use hub::{ApusHub, ServerSettings, serve, watch_routes};
pub use remote::RoutingRemote;

#[cfg(test)]
mod tests;
