//! NATS implementations of the `taurus_core::runtime::remote::RemoteRuntime`
//! and `taurus_core::runtime::engine::RespondEmitter` traits. `taurus-core`
//! has no dependency on this crate or on NATS at all; wiring these
//! implementations into an `ExecutionEngine` run is entirely up to the
//! binary crate (`taurus`, `taurus-manual`) that constructs them.

pub mod providers;
