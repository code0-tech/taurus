//! Executor for flow node execution.
//!
//! Execution model overview:
//! - The executor walks a linear "next" chain starting from `starting_node_id`.
//! - Each node can call into other nodes through lazy arguments.
//! - A node marked as remote is executed via `RemoteRuntime`.
//! - The executor is synchronous; remote calls are awaited via `block_on`.
//!
//! Remote execution:
//! - A node is considered remote based on `is_remote(&node)`.
//! - Remote args are fully resolved to concrete `Value`s before sending.
//! - The request parameters are mapped by `runtime_parameter_id`.
//! - Remote responses are mapped into `Signal::Success` or `Signal::Failure`.
//!
//! Tracing:
//! - Each node execution produces a trace frame with arguments and outcome.
//! - Child executions are linked with `EdgeKind` to reflect eager or runtime calls.
//!
//! Error behavior:
//! - Missing nodes/functions yield `Signal::Failure`.
//! - Remote failures are mapped to `RuntimeError`.
//! - The executor commits all final outcomes into the `Context`.

use crate::context::argument::{Argument, ParameterNode};
use crate::context::context::{Context, ContextResult};
use crate::context::registry::{FunctionStore, HandlerFunctionEntry};
use crate::context::signal::Signal;
use crate::debug::trace::{ArgKind, ArgTrace, EdgeKind, Outcome, ReferenceKind};
use crate::debug::tracer::{ExecutionTracer, Tracer};
use crate::runtime::error::RuntimeError;
use crate::runtime::remote::RemoteRuntime;

use futures_lite::future::block_on;
use std::collections::HashMap;
use tucana::aquila::ExecutionRequest;
use tucana::shared::reference_value::Target;
use tucana::shared::value::Kind;
use tucana::shared::{NodeFunction, Struct, Value};
use uuid::Uuid;

mod arguments;
mod execution_loop;
mod formatting;
mod node_execution;

use formatting::{preview_reference, preview_value};

#[cfg(test)]
mod tests;

/// Executes a flow graph by repeatedly evaluating nodes.
///
/// The executor is intentionally stateless with respect to the runtime:
/// it borrows the function registry and graph, and mutates only the `Context`.
pub struct Executor<'a> {
    // Registered Runtime Functions
    functions: &'a FunctionStore,
    // Nodes to execute
    nodes: HashMap<i64, NodeFunction>,
    // Connection for Remote Function Execution => Actions
    remote: Option<&'a dyn RemoteRuntime>,
    // Optional side-effect hook triggered whenever a respond signal is emitted.
    respond_emitter: Option<&'a dyn Fn(Value)>,
}

/// Determines whether a node should be executed remotely.
///
/// The current policy treats any node whose `definition_source` is not `"taurus"`
/// as a remote node.
fn is_remote(node: &NodeFunction) -> bool {
    if node.definition_source.is_empty() {
        log::warn!(
            "Found empty definition source, taking runtime as origin for node id: {}",
            node.database_id
        );
        return false;
    }

    node.definition_source != "taurus"
}

impl<'a> Executor<'a> {
    /// Create a new executor for the given function store and node map.
    ///
    /// This does not attach a remote runtime. Remote nodes will error unless
    /// a runtime is provided via `with_remote_runtime`.
    pub fn new(functions: &'a FunctionStore, nodes: HashMap<i64, NodeFunction>) -> Self {
        Self {
            functions,
            nodes,
            remote: None,
            respond_emitter: None,
        }
    }

    /// Attach a remote runtime for executing nodes marked as remote.
    ///
    /// This is a builder-style method for ergonomic setup:
    /// `Executor::new(...).with_remote_runtime(&runtime)`.
    pub fn with_remote_runtime(mut self, remote: &'a dyn RemoteRuntime) -> Self {
        self.remote = Some(remote);
        self
    }

    /// Attach a callback that is invoked for every emitted respond value.
    pub fn with_respond_emitter(mut self, emitter: &'a dyn Fn(Value)) -> Self {
        self.respond_emitter = Some(emitter);
        self
    }

    /// This is now the ONLY execution entry point.
    ///
    /// - `start_node_id` is the first node in the flow.
    /// - `ctx` is mutated in-place with results and errors.
    /// - `with_trace` controls whether the trace is printed on completion.
    pub fn execute(&self, start_node_id: i64, ctx: &mut Context, with_trace: bool) -> Signal {
        let mut tracer = Tracer::new();

        let (signal, _root_frame) = self.execute_call(start_node_id, ctx, &mut tracer);

        if with_trace && let Some(run) = tracer.take_run() {
            println!("{}", crate::debug::render::render_trace(&run));
        }
        signal
    }
}
