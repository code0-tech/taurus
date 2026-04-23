//! Runtime handler registry and callable function signatures.

use crate::handler::argument::{Argument, ParameterNode};
use crate::runtime::execution::value_store::ValueStore;
use crate::runtime::functions::ALL_FUNCTION_SETS;
use crate::types::signal::Signal;
use std::collections::HashMap;

/// Handler function type.
/// - For eager params, the executor will already convert them to Argument::Eval(Value).
/// - For lazy params, the executor will pass Argument::Thunk(node_id).
/// - If a handler wants to execute a lazy arg, it calls run(node_id).
pub type HandlerFn = fn(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal;

#[derive(Clone, Copy)]
pub enum ParamSpec {
    /// All parameters are evaluated eagerly.
    AllEager(u8),
    /// Per-parameter evaluation mode.
    Explicit(&'static [ParameterNode]),
}

impl ParamSpec {
    pub fn mode_at(self, index: usize) -> ParameterNode {
        match self {
            ParamSpec::AllEager(_) => ParameterNode::Eager,
            ParamSpec::Explicit(modes) => modes.get(index).copied().unwrap_or(ParameterNode::Eager),
        }
    }
}

#[derive(Clone, Copy)]
pub struct HandlerFunctionEntry {
    /// Callable implementation.
    pub handler: HandlerFn,
    /// Evaluation strategy for the handler parameters.
    pub param_spec: ParamSpec,
}

impl HandlerFunctionEntry {
    pub const fn eager(handler: HandlerFn, param_count: u8) -> Self {
        Self {
            handler,
            param_spec: ParamSpec::AllEager(param_count),
        }
    }

    pub const fn modes(handler: HandlerFn, param_modes: &'static [ParameterNode]) -> Self {
        Self {
            handler,
            param_spec: ParamSpec::Explicit(param_modes),
        }
    }

    pub fn param_mode(&self, index: usize) -> ParameterNode {
        self.param_spec.mode_at(index)
    }
}

#[derive(Clone, Copy)]
pub struct FunctionRegistration {
    pub id: &'static str,
    pub entry: HandlerFunctionEntry,
}

impl FunctionRegistration {
    pub const fn eager(id: &'static str, handler: HandlerFn, param_count: u8) -> Self {
        Self {
            id,
            entry: HandlerFunctionEntry::eager(handler, param_count),
        }
    }

    pub const fn modes(
        id: &'static str,
        handler: HandlerFn,
        param_modes: &'static [ParameterNode],
    ) -> Self {
        Self {
            id,
            entry: HandlerFunctionEntry::modes(handler, param_modes),
        }
    }
}

/// Holds all registered handlers.
pub struct FunctionStore {
    functions: HashMap<&'static str, HandlerFunctionEntry>,
}

impl Default for FunctionStore {
    fn default() -> Self {
        let mut store = Self::new();
        for set in ALL_FUNCTION_SETS {
            store.populate(set);
        }
        store
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

    /// Register a group of handlers.
    pub fn populate(&mut self, regs: &[FunctionRegistration]) {
        for reg in regs {
            self.functions.insert(reg.id, reg.entry);
        }
    }
}
