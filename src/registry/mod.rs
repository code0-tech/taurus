use crate::{context::Context, error::RuntimeError};
use std::collections::HashMap;
use tucana::shared::Value;

pub type HandlerFn = fn(&[Value], &mut Context) -> Result<Value, RuntimeError>;

/// Holds all registered handlers.
pub struct FunctionStore {
    functions: HashMap<String, HandlerFn>,
}

impl FunctionStore {
    /// Create a new, empty store.
    pub fn new() -> Self {
        FunctionStore {
            functions: HashMap::new(),
        }
    }

    /// Look up a handler by its ID.
    pub fn get(&self, id: &str) -> Option<&HandlerFn> {
        self.functions.get(id)
    }

    /// Execute all the registration closures to populate the map.
    pub fn populate(&mut self, regs: Vec<(&'static str, HandlerFn)>) {
        for (id, func) in regs {
            self.functions.insert(id.to_string(), func);
        }
    }
}
