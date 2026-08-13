//! Compiled runtime plan model.
//!
//! A flow is compiled into index-addressable nodes to avoid repeated map lookups
//! in the hot execution loop.

use std::collections::HashMap;

use tucana::shared::{ReferenceValue, Struct, SubFlowSetting, Value};

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
    /// A literal that contains `${signature}` placeholders resolved from
    /// `references` at execution time. See `CompiledTemplate`.
    Template(CompiledTemplate),
}

/// A literal value template plus its named inline references, compiled from
/// `tucana::shared::LiteralValue`/`InlineReferenceValue`. Each reference's
/// own value is itself a full `NodeValue`, so it compiles down to a
/// `CompiledArg` -- possibly another `Template` (nested references) or a
/// `Deferred` sub-flow.
#[derive(Debug, Clone)]
pub struct CompiledTemplate {
    pub value: Value,
    pub references: Vec<CompiledTemplateReference>,
}

#[derive(Debug, Clone)]
pub struct CompiledTemplateReference {
    pub signature: String,
    pub arg: Box<CompiledArg>,
}

#[derive(Debug, Clone)]
pub enum CompiledThunk {
    Node {
        node_id: i64,
        /// Declared shape of the sub-flow's input/output, carried from
        /// `shared::SubFlow` so a remote action call that mints this
        /// reference into a wire-format `ActionNodeSubFlowValue` can attach
        /// them without a second lookup (see `resolve_remote_args`).
        input_schema: Option<Struct>,
        output_schema: Option<Struct>,
    },
    Function {
        identifier: String,
        execution_target: NodeExecutionTarget,
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
