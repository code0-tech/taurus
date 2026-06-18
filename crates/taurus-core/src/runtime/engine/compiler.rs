//! Flow compiler for runtime execution plans.

use std::collections::HashMap;

use tucana::shared::{NodeFunction, node_value, sub_flow};

use crate::{
    runtime::engine::model::{
        CompiledArg, CompiledFlow, CompiledNode, CompiledParameter, CompiledThunk,
        NodeExecutionTarget,
    },
    types::errors::runtime_error::RuntimeError,
};

#[derive(Debug)]
pub enum CompileError {
    NodeIdMissing {
        node_index: usize,
    },
    DuplicateNodeId {
        node_id: i64,
    },
    StartNodeMissing {
        node_id: i64,
    },
    NextNodeMissing {
        node_id: i64,
        next_node_id: i64,
    },
    ParameterValueMissing {
        node_id: i64,
        parameter_index: usize,
    },
    SubFlowExecutionReferenceMissing {
        node_id: i64,
        parameter_index: usize,
    },
    EmptyRemoteService {
        node_id: i64,
        definition_source: String,
    },
}

impl CompileError {
    pub fn as_runtime_error(&self) -> RuntimeError {
        match self {
            CompileError::NodeIdMissing { node_index } => RuntimeError::new(
                "T-CORE-000100",
                "FlowCompileError",
                format!("Node at index {} is missing database id", node_index),
            ),
            CompileError::DuplicateNodeId { node_id } => RuntimeError::new(
                "T-CORE-000101",
                "FlowCompileError",
                format!("Duplicate node id in flow: {}", node_id),
            ),
            CompileError::StartNodeMissing { node_id } => RuntimeError::new(
                "T-CORE-000102",
                "FlowCompileError",
                format!("Start node not found in flow: {}", node_id),
            ),
            CompileError::NextNodeMissing {
                node_id,
                next_node_id,
            } => RuntimeError::new(
                "T-CORE-000103",
                "FlowCompileError",
                format!(
                    "Node {} points to missing next node {}",
                    node_id, next_node_id
                ),
            ),
            CompileError::ParameterValueMissing {
                node_id,
                parameter_index,
            } => RuntimeError::new(
                "T-CORE-000104",
                "FlowCompileError",
                format!(
                    "Node {} parameter {} does not contain a value",
                    node_id, parameter_index
                ),
            ),
            CompileError::SubFlowExecutionReferenceMissing {
                node_id,
                parameter_index,
            } => RuntimeError::new(
                "T-CORE-000105",
                "FlowCompileError",
                format!(
                    "Node {} parameter {} sub_flow is missing execution reference",
                    node_id, parameter_index
                ),
            ),
            CompileError::EmptyRemoteService {
                node_id,
                definition_source,
            } => RuntimeError::new(
                "T-CORE-000106",
                "FlowCompileError",
                format!(
                    "Node {} definition_source '{}' does not contain a remote service name",
                    node_id, definition_source
                ),
            ),
        }
    }
}

pub fn compile_flow(
    start_node_id: i64,
    nodes: Vec<NodeFunction>,
) -> Result<CompiledFlow, CompileError> {
    let mut node_idx_by_id = HashMap::with_capacity(nodes.len());
    for (idx, node) in nodes.iter().enumerate() {
        let Some(node_id) = node.database_id else {
            return Err(CompileError::NodeIdMissing { node_index: idx });
        };

        if node_idx_by_id.insert(node_id, idx).is_some() {
            return Err(CompileError::DuplicateNodeId { node_id });
        }
    }

    let start_idx = match node_idx_by_id.get(&start_node_id).copied() {
        Some(idx) => idx,
        None => {
            return Err(CompileError::StartNodeMissing {
                node_id: start_node_id,
            });
        }
    };

    let mut compiled_nodes = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node_id = node
            .database_id
            .expect("compiler validates node database ids before compilation");
        let next_idx = match node.next_node_id {
            Some(next_id) => match node_idx_by_id.get(&next_id).copied() {
                Some(idx) => Some(idx),
                None => {
                    return Err(CompileError::NextNodeMissing {
                        node_id,
                        next_node_id: next_id,
                    });
                }
            },
            None => None,
        };

        let execution_target = execution_target_for(node_id, &node)?;

        let mut parameters = Vec::with_capacity(node.parameters.len());
        for (parameter_index, parameter) in node.parameters.iter().enumerate() {
            let Some(node_value) = parameter.value.as_ref() else {
                return Err(CompileError::ParameterValueMissing {
                    node_id,
                    parameter_index,
                });
            };
            let Some(value) = node_value.value.as_ref() else {
                return Err(CompileError::ParameterValueMissing {
                    node_id,
                    parameter_index,
                });
            };

            let arg = match value {
                node_value::Value::LiteralValue(v) => CompiledArg::Literal(v.clone()),
                node_value::Value::ReferenceValue(r) => CompiledArg::Reference(r.clone()),
                node_value::Value::SubFlow(sub_flow) => {
                    match sub_flow.execution_reference.as_ref() {
                        Some(sub_flow::ExecutionReference::StartingNodeId(node_id)) => {
                            CompiledArg::Deferred(CompiledThunk::Node(*node_id))
                        }
                        Some(sub_flow::ExecutionReference::FunctionIdentifier(identifier)) => {
                            CompiledArg::Deferred(CompiledThunk::Function {
                                identifier: identifier.clone(),
                                parameter_index: parameter_index as i64,
                                settings: sub_flow.settings.clone(),
                            })
                        }
                        None => {
                            return Err(CompileError::SubFlowExecutionReferenceMissing {
                                node_id,
                                parameter_index,
                            });
                        }
                    }
                }
            };

            parameters.push(CompiledParameter {
                runtime_parameter_id: parameter.runtime_parameter_id.clone(),
                arg,
            });
        }

        compiled_nodes.push(CompiledNode {
            id: node_id,
            handler_id: node.runtime_function_id,
            execution_target,
            next_idx,
            parameters,
        });
    }

    Ok(CompiledFlow {
        start_idx,
        nodes: compiled_nodes,
        node_idx_by_id,
    })
}

fn execution_target_for(
    node_id: i64,
    node: &NodeFunction,
) -> Result<NodeExecutionTarget, CompileError> {
    match node.definition_source.as_deref() {
        None | Some("") | Some("taurus") => Ok(NodeExecutionTarget::Local),
        Some(source) if source.starts_with("draco") => Ok(NodeExecutionTarget::Local),
        Some(service) => match service.strip_prefix("action.").unwrap_or(service) {
            "" => Err(CompileError::EmptyRemoteService {
                node_id,
                definition_source: service.to_string(),
            }),
            service => Ok(NodeExecutionTarget::Remote {
                service: service.to_string(),
            }),
        },
    }
}
