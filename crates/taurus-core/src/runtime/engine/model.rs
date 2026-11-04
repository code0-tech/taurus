//! Compiled runtime plan model.
//!
//! A flow is compiled into index-addressable nodes to avoid repeated map lookups
//! in the hot execution loop.

use std::collections::HashMap;

use tucana::shared::{ReferenceValue, Struct, Value};

/// One parameter of a function sub-flow (`SubFlowParameters`), compiled like a
/// node parameter.
///
/// `default` is the fully compiled parameter value, used when the caller
/// supplies nothing (or a null) for this parameter. Because it is compiled
/// through the same path as a node parameter (`compiler::compile_node_value`),
/// it may be a literal, a reference, a `${signature}` template, or a nested
/// sub-flow -- not only a bare literal as before. `None` when the parameter
/// declared no value.
///
/// Note: `SubFlowParameters.cast` is not applied, matching node parameters --
/// casting is not implemented anywhere in the engine yet.
#[derive(Debug, Clone)]
pub struct SubFlowSetting {
    pub identifier: String,
    pub default: Option<CompiledArg>,
}

#[derive(Debug, Clone)]
pub enum NodeExecutionTarget {
    Local,
    Remote { service: String },
}

impl NodeExecutionTarget {
    /// Resolves where a function with the given `definition_source` runs:
    /// Taurus' own functions locally, and a function an action contributed
    /// (`action.<service>`) remotely on that action.
    ///
    /// The `action.` prefix is mandatory for a remote target. A source that
    /// names no service -- anything that is neither local nor a well-formed
    /// `action.<service>`, including a bare `"action"` with no identifier --
    /// returns `None`, so the caller fails loudly (`T-CORE-000008`) instead of
    /// dispatching a call to a service that doesn't exist.
    pub fn for_definition_source(definition_source: Option<&str>) -> Option<Self> {
        match definition_source {
            None | Some("") | Some("taurus") => Some(Self::Local),
            Some(source) => match source.strip_prefix("action.") {
                None | Some("") => None,
                Some(service) => Some(Self::Remote {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_sources_resolve_to_local() {
        assert!(matches!(
            NodeExecutionTarget::for_definition_source(None),
            Some(NodeExecutionTarget::Local)
        ));
        assert!(matches!(
            NodeExecutionTarget::for_definition_source(Some("")),
            Some(NodeExecutionTarget::Local)
        ));
        assert!(matches!(
            NodeExecutionTarget::for_definition_source(Some("taurus")),
            Some(NodeExecutionTarget::Local)
        ));
    }

    #[test]
    fn action_prefixed_source_resolves_to_its_service() {
        match NodeExecutionTarget::for_definition_source(Some("action.send-email")) {
            Some(NodeExecutionTarget::Remote { service }) => assert_eq!(service, "send-email"),
            other => panic!("expected remote send-email, got {other:?}"),
        }
    }

    #[test]
    fn bare_action_names_no_service() {
        // The failure behind code0-tech/aquila#466: hercules' raw "action"
        // must not resolve to a service called "action".
        assert!(NodeExecutionTarget::for_definition_source(Some("action")).is_none());
    }

    #[test]
    fn action_prefix_without_identifier_names_no_service() {
        assert!(NodeExecutionTarget::for_definition_source(Some("action.")).is_none());
    }

    #[test]
    fn unprefixed_source_is_not_silently_remote() {
        // Without the mandatory `action.` prefix the source names no service,
        // rather than being dispatched to a bare `<service>` as before.
        assert!(NodeExecutionTarget::for_definition_source(Some("send-email")).is_none());
    }
}
