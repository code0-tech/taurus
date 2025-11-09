use crate::context::Context;
use crate::context::argument::{Argument, ParameterNode};
use crate::context::signal::Signal;
use std::collections::HashMap;

/// HandlerFm
/// - For eager params, the executor will already convert them to Argument::Eval(Value).
/// - For lazy params, the executor will pass Argument::Thunk(node_id).
/// - If a handler wants to execute a lazy arg, it calls run(node_id).
pub type HandlerFn =
    fn(args: &[Argument], ctx: &mut Context, run: &mut dyn FnMut(i64) -> Signal) -> Signal;

pub struct HandlerFunctionEntry {
    pub handler: HandlerFn,
    pub param_modes: Vec<ParameterNode>,
}

/// Holds all registered handlers.
pub struct FunctionStore {
    functions: HashMap<String, HandlerFunctionEntry>,
}

impl Default for FunctionStore {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IntoFunctionEntry {
    fn into_function_entry(self, param: Vec<ParameterNode>) -> HandlerFunctionEntry;
    fn eager(self, param_amount: i8) -> HandlerFunctionEntry;
    fn lazy(self, param_amount: i8) -> HandlerFunctionEntry;
}

impl IntoFunctionEntry for HandlerFn {
    fn into_function_entry(self, param: Vec<ParameterNode>) -> HandlerFunctionEntry {
        HandlerFunctionEntry {
            handler: self,
            param_modes: param,
        }
    }

    fn eager(self, param_amount: i8) -> HandlerFunctionEntry {
        let mut params = vec![];

        for _ in 0..param_amount {
            params.push(ParameterNode::Eager)
        }

        HandlerFunctionEntry {
            handler: self,
            param_modes: params,
        }
    }

    fn lazy(self, param_amount: i8) -> HandlerFunctionEntry {
        let mut params = vec![];

        for _ in 0..param_amount {
            params.push(ParameterNode::Lazy)
        }

        HandlerFunctionEntry {
            handler: self,
            param_modes: params,
        }
    }
}

impl FunctionStore {
    /// Create a new, empty store.
    pub fn new() -> Self {
        FunctionStore {
            functions: HashMap::new(),
        }
    }

    /// Look up a handler by its ID.
    pub fn get(&self, id: &str) -> Option<&HandlerFunctionEntry> {
        self.functions.get(id)
    }

    /// Execute all the registration closures to populate the map.
    pub fn populate(&mut self, regs: Vec<(&'static str, HandlerFunctionEntry)>) {
        for (id, func) in regs {
            self.functions.insert(id.to_string(), func);
        }
    }
}
