//! Compiled runtime plan model.
//!
//! A flow is compiled into index-addressable nodes to avoid repeated map lookups
//! in the hot execution loop.

use std::collections::HashMap;

use tucana::shared::{ReferenceValue, SubFlowSetting, Value};

#[derive(Debug, Clone)]
pub enum NodeExecutionTarget {
    Local,
    Remote { service: String },
}

/// Argument expression compiled from proto node parameter values.
#[derive(Debug, Clone)]
pub enum CompiledArg {
    Literal(Value),
    Reference(ReferenceValue),
    Deferred(CompiledThunk),
}

#[derive(Debug, Clone)]
pub enum CompiledThunk {
    Node(i64),
    Function {
        identifier: String,
        parameter_index: i64,
        settings: Vec<SubFlowSetting>,
    },
}

/// Compiled parameter binding.
#[derive(Debug, Clone)]
pub struct CompiledParameter {
    pub runtime_parameter_id: String,
    pub arg: CompiledArg,
}

/// Compiled node representation.
#[derive(Debug, Clone)]
pub struct CompiledNode {
    pub id: i64,
    pub handler_id: String,
    pub execution_target: NodeExecutionTarget,
    pub next_idx: Option<usize>,
    pub parameters: Vec<CompiledParameter>,
}

/// Compiled flow plan.
#[derive(Debug, Clone)]
pub struct CompiledFlow {
    pub project_id: i64,
    pub start_idx: usize,
    pub nodes: Vec<CompiledNode>,
    pub node_idx_by_id: HashMap<i64, usize>,
}
