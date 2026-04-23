//! Argument binding and expression model for node execution.

use tucana::shared::Value;

use crate::types::execution::ids::{NodeId, ParameterId};

/// Path segment for nested lookups inside values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePathSegment {
    /// Select list element by index.
    Index(usize),
    /// Select object field by key.
    Field(String),
}

/// Source reference used by argument expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceSource {
    /// Input value of the overall flow.
    FlowInput,
    /// Output result of another node.
    NodeResult(NodeId),
    /// Runtime input slot (used by iterators/predicates).
    InputSlot {
        node_id: NodeId,
        parameter_index: i32,
        input_index: i32,
    },
}

/// Read expression from execution state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueReference {
    pub source: ReferenceSource,
    pub path: Vec<ValuePathSegment>,
}

/// Argument expression bound to a node parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentExpr {
    /// Constant value literal.
    ValueLiteral(Value),
    /// Value resolved from runtime references.
    Reference(ValueReference),
    /// Deferred execution entry point (lazy function parameter).
    DeferredCall(NodeId),
}

/// Argument binding for one parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct InputBinding {
    pub parameter_id: ParameterId,
    pub expression: ArgumentExpr,
}
