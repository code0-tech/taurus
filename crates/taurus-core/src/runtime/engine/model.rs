//! Compiled runtime plan model.
//!
//! A flow is compiled into index-addressable nodes to avoid repeated map lookups
//! in the hot execution loop.

use std::collections::HashMap;

use tucana::shared::{ReferenceValue, Struct, SubFlowParameters, Value, node_value};

/// One parameter of a function sub-flow, as the executor resolves it.
///
/// Tucana 0.0.83 replaced the wire-level `SubFlowSetting`
/// (`default_value`/`optional`/`hidden`) with `SubFlowParameters`
/// (`NodeValue` + `cast`). Until the executor resolves those natively, each
/// parameter is mapped onto the previous semantics: a plain literal value acts
/// as the default used when the caller doesn't supply one; everything else
/// (references, nested sub-flows, `cast`) is not yet honoured.
// TODO: resolve `SubFlowParameters` values and casts like node parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SubFlowSetting {
    pub identifier: String,
    pub default_value: Option<Value>,
}

impl From<&SubFlowParameters> for SubFlowSetting {
    fn from(parameter: &SubFlowParameters) -> Self {
        let default_value = match parameter
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
        {
            Some(node_value::Value::LiteralValue(literal)) if literal.references.is_empty() => {
                literal.value.clone()
            }
            _ => None,
        };

        Self {
            identifier: parameter.identifier.clone(),
            default_value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeExecutionTarget {
    Local,
    Remote { service: String },
}

impl NodeExecutionTarget {
    /// Resolves where a function with the given `definition_source` runs:
    /// Taurus' own and Draco's functions locally, everything else on the
    /// action named by the source (`action.<service>` or `<service>`).
    /// Returns `None` for a source that names no service at all.
    pub fn for_definition_source(definition_source: Option<&str>) -> Option<Self> {
        match definition_source {
            None | Some("") | Some("taurus") => Some(Self::Local),
            Some(source) if source.starts_with("draco") => Some(Self::Local),
            Some(source) => match source.strip_prefix("action.").unwrap_or(source) {
                "" => None,
                service => Some(Self::Remote {
                    service: service.to_string(),
                }),
            },
        }
    }
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
