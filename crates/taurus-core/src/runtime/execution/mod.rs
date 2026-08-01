//! Runtime execution internals.
//!
//! These types are owned by the execution engine lifecycle and are not part of
//! the transport-level flow contracts.

pub mod render;
pub mod trace;
pub mod tracer;
pub mod value_store;
