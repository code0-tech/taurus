//! Static flow graph representation used by the execution engine.

use std::collections::HashMap;

use crate::types::execution::bindings::InputBinding;
use crate::types::execution::ids::{FlowId, NodeId};

/// Node execution location kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Node executes in the local Taurus runtime.
    Local,
    /// Node executes in a remote runtime/service.
    Remote { service: String },
}

/// Node invocation metadata and wiring.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub handler_id: String,
    pub next: Option<NodeId>,
    pub bindings: Vec<InputBinding>,
}

/// Immutable flow graph passed to the executor.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowGraph {
    pub id: FlowId,
    pub start_node: NodeId,
    pub nodes: HashMap<NodeId, FlowNode>,
}
