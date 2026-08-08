//! Runtime handler registry and callable function signatures.

use crate::handler::argument::{Argument, ParameterNode, Thunk};
use crate::runtime::execution::value_store::ValueStore;
use crate::types::signal::Signal;
use std::collections::HashMap;

/// Handler function type.
/// - For eager params, the executor will already convert them to Argument::Eval(Value).
/// - For lazy params, the executor will pass Argument::Thunk(thunk).
/// - If a handler wants to execute a lazy arg, it calls run(thunk).
pub type ThunkRunner<'runner> = dyn FnMut(&Thunk, &mut ValueStore) -> Signal + 'runner;

pub type HandlerFn = for<'runner> fn(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut ThunkRunner<'runner>,
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

// Populated by `#[taurus_macros::runtime_function(...)]` via
// `inventory::submit!`.
inventory::collect!(FunctionRegistration);

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
        for reg in inventory::iter::<FunctionRegistration>() {
            store.functions.insert(reg.id, reg.entry);
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

    /// Register a group of handlers. Only used by tests to inject
    /// test-only handlers without polluting the global inventory registry.
    #[cfg(test)]
    pub fn populate(&mut self, regs: &[FunctionRegistration]) {
        for reg in regs {
            self.functions.insert(reg.id, reg.entry);
        }
    }
}
