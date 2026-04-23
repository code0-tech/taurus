//! Runtime execution surfaces.
//!
//! `engine` is the public execution API. The remaining modules contain runtime
//! internals and transport-specific abstractions used by the engine.

pub mod engine;
pub mod execution;
pub mod functions;
pub mod remote;
