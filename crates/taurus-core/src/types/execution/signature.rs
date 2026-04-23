//! Handler/function signature model for runtime execution.

use crate::types::execution::ids::ParameterId;

/// How a parameter argument should be evaluated by the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationMode {
    /// Argument is resolved before invoking the handler.
    Eager,
    /// Argument is provided as deferred executable call/input expression.
    Deferred,
}

/// Single parameter contract for a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterSpec {
    pub id: ParameterId,
    pub name: String,
    pub evaluation_mode: EvaluationMode,
    pub required: bool,
}

/// Complete handler contract used by registry and runtime checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerSignature {
    pub handler_id: String,
    pub parameters: Vec<ParameterSpec>,
}
