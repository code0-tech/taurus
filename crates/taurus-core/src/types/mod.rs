//! Shared runtime domain types.
//!
//! Split by concern: execution model, signal vocabulary, and error contracts.

pub mod errors;
pub mod execution;
pub mod exit_reason;
pub mod signal;
