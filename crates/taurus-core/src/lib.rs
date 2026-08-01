//! Taurus core runtime library: the transport-agnostic flow execution engine.
//! Nothing here depends on NATS, gRPC, or Tokio's networking types, so it can
//! be driven synchronously (`taurus-tests`) or wired to a live transport
//! (`taurus`, `taurus-manual` via `taurus-provider`) without changes.
//!
//! See [`runtime::engine`] for the public execution API, [`types`] for the
//! shared signal/error vocabulary, and [`fixtures`]/[`normalize`] for the
//! JSON fixture and proto-value-normalization helpers shared by the runtime
//! binaries.

pub mod fixtures;
mod handler;
pub mod normalize;
pub mod runtime;
pub mod time;
pub mod types;
pub mod value;
