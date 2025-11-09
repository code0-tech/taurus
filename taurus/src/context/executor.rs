use crate::context::Context;
use crate::context::argument::{Argument, ParameterNode};
use crate::context::registry::FunctionStore;
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use std::cell::RefCell;
use std::collections::HashMap;
use tucana::shared::{NodeFunction};

pub struct Executor<'a> {
    functions: &'a FunctionStore,
    nodes: HashMap<i64, NodeFunction>,
    context: RefCell<Context>,
}

impl<'a> Executor<'a> {
    pub fn new(
        functions: &'a FunctionStore,
        nodes: HashMap<i64, NodeFunction>,
        context: Context,
    ) -> Self {
        Executor {
            functions,
            nodes,
            context: RefCell::new(context),
        }
    }

    pub fn execute(&self, starting_node_id: i64) -> Signal {
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

            let entry = match self.functions.get(node.runtime_function_id.as_str()) {
                None => {
                    return Signal::Failure(RuntimeError::simple_str(
                        "FunctionNotFound",
                        "The function was not found",
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
                            "Missing parameter value",
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
                    tucana::shared::node_value::Value::ReferenceValue(_r) => {
                        unimplemented!("ReferenceValue")
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
                if matches!(mode, ParameterNode::Eager) {
                    if let Argument::Thunk(id) = *a {
                        match self.execute(id) {
                            Signal::Success(v) => *a = Argument::Eval(v),
                            s @ (Signal::Failure(_)
                            | Signal::Return(_)
                            | Signal::Respond(_)
                            | Signal::Stop) => return s,
                        }
                    }
                }
            }

            let mut run = |node_id: i64| self.execute(node_id);
            let mut ctx = self.context.borrow_mut();
            let result = (entry.handler)(&args, &mut ctx, &mut run);

            match result {
                Signal::Success(value) => {
                    if let Some(next_node_id) = node.next_node_id {
                        current_node_id = next_node_id;
                        continue;
                    } else {
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
