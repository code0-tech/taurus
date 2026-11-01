//! Clients Taurus uses to talk *to* Aquila in dynamic mode: execution
//! results ([`runtime_execution`]) and runtime status/heartbeats
//! ([`runtime_status`]) over gRPC, and per-execution runtime usage
//! ([`runtime_usage`]) group-committed into a NATS JetStream stream. All are
//! no-ops from the caller's perspective in static mode, since [`crate::app`]
//! only constructs them when `MODE=dynamic`.

pub mod runtime_execution;
pub mod runtime_status;
pub mod runtime_usage;
