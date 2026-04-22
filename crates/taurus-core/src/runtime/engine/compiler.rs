//! Flow compiler for runtime execution plans.

use std::collections::HashMap;

use tucana::shared::{NodeFunction, node_value};

use crate::{
    runtime::engine::model::{
        CompiledArg, CompiledFlow, CompiledNode, CompiledParameter, NodeExecutionTarget,
    },
    types::errors::runtime_error::RuntimeError,
};

#[derive(Debug)]
pub enum CompileError {
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
}

impl CompileError {
    pub fn as_runtime_error(&self) -> RuntimeError {
        match self {
            CompileError::DuplicateNodeId { node_id } => RuntimeError::new(
                "T-RT-000000",
                "FlowCompileError",
                format!("Duplicate node id in flow: {}", node_id),
            ),
            CompileError::StartNodeMissing { node_id } => RuntimeError::new(
                "T-RT-000000",
                "FlowCompileError",
                format!("Start node not found in flow: {}", node_id),
            ),
            CompileError::NextNodeMissing {
                node_id,
                next_node_id,
            } => RuntimeError::new(
                "T-RT-000000",
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
                "T-RT-000000",
                "FlowCompileError",
                format!(
                    "Node {} parameter {} does not contain a value",
                    node_id, parameter_index
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
        if node_idx_by_id.insert(node.database_id, idx).is_some() {
            return Err(CompileError::DuplicateNodeId {
                node_id: node.database_id,
            });
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
        let next_idx = match node.next_node_id {
            Some(next_id) => match node_idx_by_id.get(&next_id).copied() {
                Some(idx) => Some(idx),
                None => {
                    return Err(CompileError::NextNodeMissing {
                        node_id: node.database_id,
                        next_node_id: next_id,
                    });
                }
            },
            None => None,
        };

        let execution_target = execution_target_for(&node);

        let mut parameters = Vec::with_capacity(node.parameters.len());
        for (parameter_index, parameter) in node.parameters.iter().enumerate() {
            let Some(node_value) = parameter.value.as_ref() else {
                return Err(CompileError::ParameterValueMissing {
                    node_id: node.database_id,
                    parameter_index,
                });
            };
            let Some(value) = node_value.value.as_ref() else {
                return Err(CompileError::ParameterValueMissing {
                    node_id: node.database_id,
                    parameter_index,
                });
            };

            let arg = match value {
                node_value::Value::LiteralValue(v) => CompiledArg::Literal(v.clone()),
                node_value::Value::ReferenceValue(r) => CompiledArg::Reference(r.clone()),
                node_value::Value::NodeFunctionId(id) => CompiledArg::DeferredNode(*id),
            };

            parameters.push(CompiledParameter {
                runtime_parameter_id: parameter.runtime_parameter_id.clone(),
                arg,
            });
        }

        compiled_nodes.push(CompiledNode {
            id: node.database_id,
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

fn execution_target_for(node: &NodeFunction) -> NodeExecutionTarget {
    if node.definition_source.is_empty()
        || node.definition_source == "taurus"
        || node.definition_source.starts_with("draco")
    {
        NodeExecutionTarget::Local
    } else {
        NodeExecutionTarget::Remote {
            service: node.definition_source.clone(),
        }
    }
}
