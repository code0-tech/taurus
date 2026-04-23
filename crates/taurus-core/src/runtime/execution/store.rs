//! Mutable execution state for a single flow run.

use std::collections::HashMap;

use tucana::shared::Value;

use crate::types::errors::runtime_error::RuntimeError;
use crate::types::execution::ids::{FrameId, NodeId};

/// Runtime outcome persisted per node.
#[derive(Debug, Clone)]
pub enum NodeOutcome {
    Success(Value),
    Failure(RuntimeError),
}

/// Input slot key for runtime-provided temporary inputs (for iterators/predicates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputSlotKey {
    pub node_id: NodeId,
    pub parameter_index: i32,
    pub input_index: i32,
}

/// Store that captures mutable runtime execution state.
#[derive(Debug, Clone, Default)]
pub struct ExecutionStore {
    pub node_outcomes: HashMap<NodeId, NodeOutcome>,
    pub input_slots: HashMap<InputSlotKey, Value>,
    pub flow_input: Option<Value>,
    pub current_node: Option<NodeId>,
    pub frame_stack: Vec<FrameId>,
}
