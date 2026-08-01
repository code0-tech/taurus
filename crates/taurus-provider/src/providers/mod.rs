//! The two NATS-backed provider implementations: [`remote`] for delegating
//! node execution to remote services, [`emitter`] for streaming execution
//! lifecycle events.

pub mod emitter;
pub mod remote;
