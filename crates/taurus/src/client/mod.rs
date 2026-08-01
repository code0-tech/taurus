//! gRPC clients Taurus uses to talk *to* Aquila in dynamic mode: execution
//! results ([`runtime_execution`]) and runtime status/heartbeats
//! ([`runtime_status`]). Both are no-ops from the caller's perspective in
//! static mode, since [`crate::app`] only constructs them when
//! `MODE=dynamic`.

pub mod runtime_execution;
pub mod runtime_status;
