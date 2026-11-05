//! Runtime handler registry and callable function signatures.

use crate::handler::argument::{Argument, ParameterNode, Thunk};
use crate::runtime::execution::value_store::ValueStore;
use crate::types::signal::Signal;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// The future a handler (and a thunk run) returns. Boxed so a plain `fn`
/// pointer can name it, and borrows its inputs for `'a` so the body can hold
/// `args`/`ctx`/`run` across `.await` points. `Send` because the engine's
/// execution future is spawned on the multi-thread runtime (the
/// `sub_flow_execution.*` worker), so every future it awaits must be `Send`.
pub type HandlerFuture<'a> = Pin<Box<dyn Future<Output = Signal> + Send + 'a>>;

/// Re-entry point handed to a handler so it can run its lazy args and callback
/// bodies (e.g. `if`/`if_else` branches, `for_each`/`map` consumers) back
/// through the executor.
///
/// Async on purpose: a callback body that reaches a remote node now `.await`s
/// it rather than blocking the worker thread via `block_on` (code0-tech/taurus#347).
/// A trait object instead of a boxed closure so the returned future can borrow
/// `thunk`/`ctx` for a single, unified lifetime.
pub trait AsyncThunkRunner {
    fn run_thunk<'a>(&'a mut self, thunk: &'a Thunk, ctx: &'a mut ValueStore) -> HandlerFuture<'a>;
}

/// Handler callback type.
/// - For eager params, the executor will already convert them to Argument::Eval(Value).
/// - For lazy params, the executor will pass Argument::Thunk(thunk).
/// - If a handler wants to execute a lazy arg, it calls `run.run_thunk(thunk, ctx).await`.
///
/// Kept as a type alias so handler signatures still spell the runner as
/// `&mut ThunkRunner<'_>`; it is now a trait object rather than a boxed closure.
/// `Send` so a handler future that captures `&mut run` stays `Send`.
pub type ThunkRunner<'runner> = dyn AsyncThunkRunner + Send + 'runner;

pub type HandlerFn = for<'a> fn(
    args: &'a [Argument],
    ctx: &'a mut ValueStore,
    run: &'a mut ThunkRunner<'a>,
) -> HandlerFuture<'a>;

/// Adapts a synchronous `FnMut(&Thunk, &mut ValueStore) -> Signal` into an
/// [`AsyncThunkRunner`]. Convenient where a callback never needs to await
/// (e.g. tests, or a runner that returns canned results).
pub struct FnThunkRunner<F>(pub F);

impl<F> AsyncThunkRunner for FnThunkRunner<F>
where
    F: FnMut(&Thunk, &mut ValueStore) -> Signal,
{
    fn run_thunk<'a>(&'a mut self, thunk: &'a Thunk, ctx: &'a mut ValueStore) -> HandlerFuture<'a> {
        let signal = (self.0)(thunk, ctx);
        Box::pin(async move { signal })
    }
}

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
