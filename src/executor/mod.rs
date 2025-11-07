use crate::context::Context;
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use crate::registry::FunctionStore;
use std::collections::HashMap;
use tucana::shared::{NodeFunction, NodeParameter};

pub struct Executor<'a> {
    functions: &'a FunctionStore,
    nodes: HashMap<i64, NodeFunction>,
    context: Context,
}

type HandleNodeParameterFn = fn(&mut Executor, node_parameter: &NodeParameter) -> Signal;

impl<'a> Executor<'a> {
    pub fn new(
        functions: &'a FunctionStore,
        nodes: HashMap<i64, NodeFunction>,
        context: Context,
    ) -> Self {
        Executor {
            functions,
            nodes,
            context,
        }
    }

    pub fn execute(&mut self, starting_node_id: i64) -> Signal {
        let mut current_node_id = starting_node_id;

        loop {
            let node = match self.nodes.get(&current_node_id) {
                None => {
                    return Signal::Failure(RuntimeError::simple_str(
                        "NodeNotFound",
                        "The node with the id was not found",
                    ));
                }
                Some(n) => n.clone(),
            };

            let mut parameters = Vec::new();

            for node_parameter in &node.parameters {
                match Executor::handle_node_parameter(self, node_parameter) {
                    Signal::Success(value) | Signal::Return(value) => {
                        parameters.push(value.clone())
                    }
                    Signal::Failure(error) => return Signal::Failure(error),
                    Signal::Stop => return Signal::Stop,
                    Signal::Respond(value) => return Signal::Respond(value),
                }
            }

            let execution_result = match self.functions.get(node.runtime_function_id.as_str()) {
                Some(handler) => handler(&parameters, &mut self.context),
                None => {
                    return Signal::Failure(RuntimeError::simple_str(
                        "FunctionNotFound",
                        "The function was not found",
                    ));
                }
            };

            match execution_result {
                Signal::Success(value) => {
                    if let Some(next_node_id) = node.next_node_id {
                        current_node_id = next_node_id;
                        continue;
                    } else {
                        return Signal::Success(value);
                    }
                }
                Signal::Failure(runtime_error) => return Signal::Failure(runtime_error),
                Signal::Return(value) => return Signal::Return(value),
                Signal::Respond(value) => return Signal::Respond(value),
                Signal::Stop => return Signal::Stop,
            }
        }
    }

    fn handle_node_parameter(&mut self, node_parameter: &NodeParameter) -> Signal {
        let node_value = match &node_parameter.value {
            Some(v) => v,
            None => {
                return Signal::Failure(RuntimeError::simple_str(
                    "NodeValueNotFound",
                    "An error occurred while executing a flow!",
                ));
            }
        };

        let value = match &node_value.value {
            Some(v) => v,
            None => {
                return Signal::Failure(RuntimeError::simple_str(
                    "NodeValueNotFound",
                    "An error occurred while executing a flow!",
                ));
            }
        };

        match value {
            tucana::shared::node_value::Value::LiteralValue(value) => {
                Signal::Success(value.clone())
            }
            tucana::shared::node_value::Value::ReferenceValue(_reference_value) => {
                unimplemented!("implement reference values!")
            }
            tucana::shared::node_value::Value::NodeFunctionId(id) => Executor::execute(self, *id),
        }
    }
}
