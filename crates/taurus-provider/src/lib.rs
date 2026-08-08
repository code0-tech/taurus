//! NATS implementation of the `taurus_core::runtime::remote::RemoteRuntime`
//! trait. `taurus-core` has no dependency on this crate or on NATS at all;
//! wiring the adapter into an `ExecutionEngine` run is entirely up to the
//! binary crate (`taurus`, `taurus-manual`) that constructs it.

pub mod providers;
