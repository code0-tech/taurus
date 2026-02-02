use crate::context::argument::{Argument, ParameterNode};
use crate::context::context::{Context, ContextResult};
use crate::context::registry::FunctionStore;
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use std::collections::HashMap;
use tucana::shared::NodeFunction;

pub struct Executor<'a> {
    functions: &'a FunctionStore,
    nodes: HashMap<i64, NodeFunction>,
}

impl<'a> Executor<'a> {
    pub fn new(functions: &'a FunctionStore, nodes: HashMap<i64, NodeFunction>) -> Self {
        Executor { functions, nodes }
    }

    pub fn execute(&self, starting_node_id: i64, ctx: &mut Context) -> Signal {
        let mut current_node_id = starting_node_id;

        loop {
            let node = match self.nodes.get(&current_node_id) {
                None => {
                    return Signal::Failure(RuntimeError::simple(
                        "NodeNotFound",
                        format!(
                            "The node with the database id: {} was not found",
                            current_node_id
                        ),
                    ));
                }
                Some(n) => n.clone(),
            };

            let entry = match self.functions.get(node.runtime_function_id.as_str()) {
                None => {
                    return Signal::Failure(RuntimeError::simple(
                        "FunctionNotFound",
                        format!(
                            "The function {} (database id: {}) was not found",
                            node.runtime_function_id, node.database_id
                        ),
                    ));
                }
                Some(f) => f,
            };

            let mut args: Vec<Argument> = Vec::with_capacity(node.parameters.len());
            for parameter in &node.parameters {
                let node_value = match &parameter.value {
                    Some(v) => v,
                    None => {
                        return Signal::Failure(RuntimeError::simple_str(
                            "NodeValueNotFound",
                            "Missing parameter value: {}",
                        ));
                    }
                };
                let value = match &node_value.value {
                    Some(v) => v,
                    None => {
                        return Signal::Failure(RuntimeError::simple_str(
                            "NodeValueNotFound",
                            "Missing inner value",
                        ));
                    }
                };

                match value {
                    tucana::shared::node_value::Value::LiteralValue(val) => {
                        args.push(Argument::Eval(val.clone()))
                    }
                    tucana::shared::node_value::Value::ReferenceValue(reference) => {
                        let value = ctx.get(reference.node_id);
                        match value {
                            ContextResult::Error(runtime_error) => {
                                return Signal::Failure(runtime_error);
                            }
                            ContextResult::Success(result) => {
                                args.push(Argument::Eval(result.clone()));
                            }
                            ContextResult::NotFound => {
                                return Signal::Failure(RuntimeError::simple_str(
                                    "ReferenceValueNotFound",
                                    "The given node has not been executed but referenced.",
                                ));
                            }
                        }
                    }
                    tucana::shared::node_value::Value::NodeFunctionId(id) => {
                        args.push(Argument::Thunk(*id))
                    }
                }
            }

            for (i, a) in args.iter_mut().enumerate() {
                let mode = entry
                    .param_modes
                    .get(i)
                    .copied()
                    .unwrap_or(ParameterNode::Eager);
                if matches!(mode, ParameterNode::Eager)
                    && let Argument::Thunk(id) = *a
                {
                    match self.execute(id, ctx) {
                        Signal::Success(v) => {
                            log::debug!(
                                "Successfully executed node with database id {}, resulted in value: {:?}",
                                id,
                                a
                            );
                            *a = Argument::Eval(v)
                        }
                        Signal::Failure(err) => {
                            log::error!("Failed to execute node with database id: {}", id);
                            return Signal::Failure(err);
                        }
                        s @ (Signal::Return(_) | Signal::Respond(_) | Signal::Stop) => return s,
                    }
                }
            }

            let mut run = |node_id: i64, ctx: &mut Context| self.execute(node_id, ctx);
            let result = (entry.handler)(&args, ctx, &mut run);

            match result {
                Signal::Success(value) => {
                    if let Some(next_node_id) = node.next_node_id {
                        current_node_id = next_node_id;
                        continue;
                    } else {
                        log::debug!(
                            "Successfully executed node with database id {}, resulted in value: {:?}",
                            current_node_id,
                            value
                        );
                        return Signal::Success(value);
                    }
                }
                Signal::Failure(e) => return Signal::Failure(e),
                Signal::Return(v) => return Signal::Return(v),
                Signal::Respond(v) => return Signal::Respond(v),
                Signal::Stop => return Signal::Stop,
            }
        }
    }
}
