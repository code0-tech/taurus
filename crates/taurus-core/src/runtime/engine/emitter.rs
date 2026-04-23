//! Respond emitter abstraction used by the engine.

use std::fmt::{Display, Formatter};

use tucana::shared::Value;
use uuid::Uuid;

/// Unique identifier for one top-level flow execution.
pub type ExecutionId = Uuid;

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
    fn emit(&self, execution_id: ExecutionId, emit_type: EmitType, value: Value);
}

impl<F> RespondEmitter for F
where
    F: Fn(ExecutionId, EmitType, Value) + ?Sized,
{
    fn emit(&self, execution_id: ExecutionId, emit_type: EmitType, value: Value) {
        self(execution_id, emit_type, value);
    }
}

impl Display for EmitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            EmitType::StartingExec => "started_execution",
            EmitType::OngoingExec => "ongoing_execution",
            EmitType::FinishedExec => "finished_execution",
            EmitType::FailedExec => "failed_execution",
        };

        write!(f, "{label}")
    }
}
