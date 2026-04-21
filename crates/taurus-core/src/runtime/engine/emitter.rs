//! Respond emitter abstraction used by the engine.

use tucana::shared::Value;

/// Execution lifecycle event emitted by the runtime engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitType {
    /// Top-level flow execution has started.
    StartingExec,
    /// An intermediate `Signal::Respond` was emitted during execution.
    OngoingExec,
    /// Flow execution reached a non-failure terminal state.
    FinishedExec,
    /// Flow execution ended with a runtime failure.
    FailedExec,
}

/// Callback interface for streaming execution lifecycle events.
pub trait RespondEmitter {
    fn emit(&self, emit_type: EmitType, value: Value);
}

impl<F> RespondEmitter for F
where
    F: Fn(EmitType, Value) + ?Sized,
{
    fn emit(&self, emit_type: EmitType, value: Value) {
        self(emit_type, value);
    }
}
