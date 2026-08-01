//! Taurus core runtime library: the transport-agnostic flow execution engine.
//! Nothing here depends on NATS or gRPC, so it can be driven synchronously
//! (`taurus-tests`) or wired to a live transport (`taurus`, `taurus-manual`
//! via `taurus-provider`) without changes. The one exception is a thin
//! dependency on Tokio's runtime-detection API (not its networking types):
//! the `http` handler runs its blocking call via `block_in_place` when a
//! Tokio multi-thread runtime happens to be active, and calls it directly
//! otherwise -- so callers with no runtime at all (`taurus-tests`,
//! `taurus-manual --offline`) still work unchanged.
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
pub mod version;
