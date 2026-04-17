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

    /// Main execution loop.
    ///
    /// Executes nodes one-by-one along the `next_node_id` chain until a
    /// non-success signal is produced or the chain ends.
    fn execute_call(
        &self,
        start_node_id: i64,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> (Signal, u64) {
        let mut current = start_node_id;

        let mut call_root_frame: Option<u64> = None;
        let mut previous_frame: Option<u64> = None;

        loop {
            let (signal, frame_id) = self.execute_single_node(current, ctx, tracer);

            if call_root_frame.is_none() {
                call_root_frame = Some(frame_id);
            }

            // Link linear NEXT chain
            if let Some(prev) = previous_frame {
                tracer.link_child(prev, frame_id, EdgeKind::Next);
            }
            previous_frame = Some(frame_id);

            match signal {
                Signal::Success(_) => {
                    let node = self.nodes.get(&current).unwrap();

                    if let Some(next) = node.next_node_id {
                        current = next;
                        continue;
                    }

                    return (signal, call_root_frame.unwrap());
                }

                Signal::Respond(v) => {
                    let node = self.nodes.get(&current).unwrap();

                    if let Some(emit) = self.respond_emitter {
                        emit(v.clone());
                    }

                    // Respond behaves like success for flow continuation and references.
                    ctx.insert_success(node.database_id, v.clone());

                    if let Some(next) = node.next_node_id {
                        current = next;
                        continue;
                    }

                    return (Signal::Success(v), call_root_frame.unwrap());
                }

                _ => return (signal, call_root_frame.unwrap()),
            }
        }
    }

    /// Execute a single node and return its signal and trace frame id.
    ///
    /// This handles:
    /// - Node lookup
    /// - Remote vs local dispatch
    /// - Argument building and eager evaluation
    /// - Handler invocation and result commit
    fn execute_single_node(
        &self,
        node_id: i64,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
    ) -> (Signal, u64) {
        ctx.set_current_node_id(node_id);
        let node = match self.nodes.get(&node_id) {
            Some(n) => n.clone(),
            None => {
                let err =
                    RuntimeError::simple("NodeNotFound", format!("Node {} not found", node_id));
                return (Signal::Failure(err), 0);
            }
        };

        if is_remote(&node) {
            let remote = match self.remote {
                Some(r) => r,
                None => {
                    let err = RuntimeError::simple(
                        "RemoteRuntimeNotConfigured",
                        "Remote runtime not configured".to_string(),
                    );
                    return (Signal::Failure(err), 0);
                }
            };

            let frame_id = tracer.enter_node(node.database_id, node.runtime_function_id.as_str());

            let mut args = match self.build_args(&node, ctx, tracer, frame_id) {
                Ok(a) => a,
                Err(e) => {
                    ctx.insert_error(node.database_id, e.clone());
                    tracer.exit_node(
                        frame_id,
                        Outcome::Failure {
                            error_preview: format!("{:#?}", e),
                        },
                    );
                    return (Signal::Failure(e), frame_id);
                }
            };

            let values = match self.resolve_remote_args(&mut args, ctx, tracer, frame_id) {
                Ok(v) => v,
                Err((sig, outcome)) => {
                    tracer.exit_node(frame_id, outcome);
                    return (sig, frame_id);
                }
            };

            let request = match self.build_remote_request(&node, values) {
                Ok(r) => r,
                Err(e) => {
                    ctx.insert_error(node.database_id, e.clone());
                    tracer.exit_node(
                        frame_id,
                        Outcome::Failure {
                            error_preview: format!("{:#?}", e),
                        },
                    );
                    return (Signal::Failure(e), frame_id);
                }
            };

            let remote_result =
                block_on(remote.execute_remote(node.definition_source.clone(), request));
            let signal = match remote_result {
                Ok(value) => Signal::Success(value),
                Err(err) => Signal::Failure(err),
            };

            let final_signal = self.commit_result(&node, signal, ctx, tracer, frame_id);
            return (final_signal, frame_id);
        }

        let entry = match self.functions.get(node.runtime_function_id.as_str()) {
            Some(e) => e,
            None => {
                let err = RuntimeError::simple(
                    "FunctionNotFound",
                    format!("Function {} not found", node.runtime_function_id),
                );
                return (Signal::Failure(err), 0);
            }
        };

        let frame_id = tracer.enter_node(node.database_id, node.runtime_function_id.as_str());

        // ---- Build args
        let mut args = match self.build_args(&node, ctx, tracer, frame_id) {
            Ok(a) => a,
            Err(e) => {
                ctx.insert_error(node.database_id, e.clone());
                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );
                return (Signal::Failure(e), frame_id);
            }
        };

        // ---- Force eager args
        if let Err((sig, outcome)) =
            self.force_eager_args(&node, entry, &mut args, ctx, tracer, frame_id)
        {
            tracer.exit_node(frame_id, outcome);
            return (sig, frame_id);
        }

        // ---- Invoke handler
        let result = self.invoke_handler(entry, &args, ctx, tracer, frame_id);

        // ---- Commit result
        let final_signal = self.commit_result(&node, result, ctx, tracer, frame_id);

        (final_signal, frame_id)
    }

    /// Build arguments for a node from literals, references, or thunks.
    ///
    /// Arguments are recorded to the tracer for debugging and inspection.
    /// Thunks are represented by the referenced node id.
    fn build_args(
        &self,
        node: &NodeFunction,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Result<Vec<Argument>, RuntimeError> {
        let mut args = Vec::with_capacity(node.parameters.len());

        for (i, param) in node.parameters.iter().enumerate() {
            let node_value = param.value.as_ref().ok_or_else(|| {
                RuntimeError::simple_str("NodeValueNotFound", "Missing param value")
            })?;

            let value = node_value.value.as_ref().ok_or_else(|| {
                RuntimeError::simple_str("NodeValueNotFound", "Missing inner value")
            })?;

            match value {
                tucana::shared::node_value::Value::LiteralValue(v) => {
                    tracer.record_arg(
                        frame_id,
                        ArgTrace {
                            index: i,
                            kind: ArgKind::Literal,
                            preview: preview_value(v),
                        },
                    );
                    args.push(Argument::Eval(v.clone()));
                }

                tucana::shared::node_value::Value::ReferenceValue(r) => match ctx.get(r.clone()) {
                    ContextResult::Success(v) => {
                        let reference = match r.target {
                            Some(ref_value) => match ref_value {
                                tucana::shared::reference_value::Target::FlowInput(_) => {
                                    ReferenceKind::FlowInput
                                }
                                tucana::shared::reference_value::Target::NodeId(id) => {
                                    ReferenceKind::Result { node_id: id }
                                }
                                tucana::shared::reference_value::Target::InputType(input_type) => {
                                    ReferenceKind::InputType {
                                        node_id: input_type.node_id,
                                        input_index: input_type.input_index,
                                        parameter_index: input_type.parameter_index,
                                    }
                                }
                            },
                            None => ReferenceKind::Empty,
                        };

                        tracer.record_arg(
                            frame_id,
                            ArgTrace {
                                index: i,
                                kind: ArgKind::Reference {
                                    reference,
                                    hit: true,
                                },
                                preview: format!(
                                    "ctx.get({}) -> {}",
                                    preview_reference(r),
                                    preview_value(&v)
                                ),
                            },
                        );
                        args.push(Argument::Eval(v));
                    }
                    ContextResult::Error(e) => return Err(e),
                    ContextResult::NotFound => {
                        return Err(RuntimeError::simple_str(
                            "ReferenceValueNotFound",
                            "Reference not found in context",
                        ));
                    }
                },

                tucana::shared::node_value::Value::NodeFunctionId(id) => {
                    tracer.record_arg(
                        frame_id,
                        ArgTrace {
                            index: i,
                            kind: ArgKind::Thunk {
                                node_id: *id,
                                eager: false,
                                executed: false,
                            },
                            preview: format!("thunk({})", id),
                        },
                    );
                    args.push(Argument::Thunk(*id));
                }
            }
        }

        Ok(args)
    }

    /// Eagerly execute any argument marked as `Eager`.
    ///
    /// Lazy arguments are preserved as thunks until needed by a handler.
    /// If an eager child fails, the failure bubbles up immediately.
    fn force_eager_args(
        &self,
        _node: &NodeFunction,
        entry: &HandlerFunctionEntry,
        args: &mut [Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        parent_frame: u64,
    ) -> Result<(), (Signal, Outcome)> {
        for (i, arg) in args.iter_mut().enumerate() {
            let mode = entry
                .param_modes
                .get(i)
                .copied()
                .unwrap_or(ParameterNode::Eager);

            if matches!(mode, ParameterNode::Eager)
                && let Argument::Thunk(id) = *arg
            {
                tracer.mark_thunk(parent_frame, i, true, true);
                let (child_sig, child_root) = self.execute_call(id, ctx, tracer);

                tracer.link_child(
                    parent_frame,
                    child_root,
                    EdgeKind::EagerCall { arg_index: i },
                );

                match child_sig {
                    Signal::Success(v) => {
                        *arg = Argument::Eval(v);
                    }
                    Signal::Return(v) => {
                        // `return` only exits the child context and yields a value upward.
                        *arg = Argument::Eval(v);
                    }
                    s => {
                        return Err((
                            s,
                            Outcome::Failure {
                                error_preview: "Eager child failed".into(),
                            },
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Invoke a local handler with a closure for lazy execution.
    ///
    /// The closure will evaluate a thunk node and link its trace to the
    /// current execution frame.
    fn invoke_handler(
        &self,
        entry: &HandlerFunctionEntry,
        args: &[Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Signal {
        let mut run = |node_id: i64, ctx: &mut Context| {
            tracer.mark_thunk_executed_by_node(frame_id, node_id);
            let label = ctx.pop_runtime_trace_label();
            let (sig, child_root) = self.execute_call(node_id, ctx, tracer);
            tracer.link_child(frame_id, child_root, EdgeKind::RuntimeCall { label });
            match sig {
                // `return` only exits the child context and is a value for the caller.
                Signal::Return(v) => Signal::Success(v),
                other => other,
            }
        };

        (entry.handler)(args, ctx, &mut run)
    }

    /// Persist the final signal into the context and close the trace frame.
    fn commit_result(
        &self,
        node: &NodeFunction,
        result: Signal,
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        frame_id: u64,
    ) -> Signal {
        match result {
            Signal::Success(v) => {
                ctx.insert_success(node.database_id, v.clone());

                tracer.exit_node(
                    frame_id,
                    Outcome::Success {
                        value_preview: preview_value(&v),
                    },
                );

                Signal::Success(v)
            }

            Signal::Failure(e) => {
                ctx.insert_error(node.database_id, e.clone());

                tracer.exit_node(
                    frame_id,
                    Outcome::Failure {
                        error_preview: format!("{:#?}", e),
                    },
                );

                Signal::Failure(e)
            }

            Signal::Return(v) => {
                tracer.exit_node(
                    frame_id,
                    Outcome::Return {
                        value_preview: preview_value(&v),
                    },
                );
                Signal::Return(v)
            }

            Signal::Respond(v) => {
                tracer.exit_node(
                    frame_id,
                    Outcome::Respond {
                        value_preview: preview_value(&v),
                    },
                );
                Signal::Respond(v)
            }

            Signal::Stop => {
                tracer.exit_node(frame_id, Outcome::Stop);
                Signal::Stop
            }
        }
    }

    /// Resolve all arguments for a remote call.
    ///
    /// Remote execution requires concrete values, so any thunks are executed
    /// eagerly and replaced with their resulting `Value`.
    fn resolve_remote_args(
        &self,
        args: &mut [Argument],
        ctx: &mut Context,
        tracer: &mut dyn ExecutionTracer,
        parent_frame: u64,
    ) -> Result<Vec<Value>, (Signal, Outcome)> {
        let mut values = Vec::with_capacity(args.len());

        for (i, arg) in args.iter_mut().enumerate() {
            match arg {
                Argument::Eval(v) => values.push(v.clone()),
                Argument::Thunk(id) => {
                    tracer.mark_thunk(parent_frame, i, true, true);
                    let (child_sig, child_root) = self.execute_call(*id, ctx, tracer);
                    if child_root != 0 {
                        tracer.link_child(
                            parent_frame,
                            child_root,
                            EdgeKind::EagerCall { arg_index: i },
                        );
                    }

                    match child_sig {
                        Signal::Success(v) => {
                            *arg = Argument::Eval(v.clone());
                            values.push(v);
                        }
                        Signal::Return(v) => {
                            // `return` only exits the child context and yields a value upward.
                            *arg = Argument::Eval(v.clone());
                            values.push(v);
                        }
                        s => {
                            return Err((
                                s,
                                Outcome::Failure {
                                    error_preview: "Eager child failed".into(),
                                },
                            ));
                        }
                    }
                }
            }
        }

        Ok(values)
    }

    /// Build a remote execution request from a node and its resolved values.
    ///
    /// Values are mapped to their parameter ids for the remote runtime.
    fn build_remote_request(
        &self,
        node: &NodeFunction,
        values: Vec<Value>,
    ) -> Result<ExecutionRequest, RuntimeError> {
        if node.parameters.len() != values.len() {
            return Err(RuntimeError::simple_str(
                "RemoteParameterMismatch",
                "Remote parameter count mismatch",
            ));
        }

        let mut fields = HashMap::new();
        for (param, value) in node.parameters.iter().zip(values.into_iter()) {
            fields.insert(param.runtime_parameter_id.clone(), value);
        }
        let id = Uuid::new_v4();
        Ok(ExecutionRequest {
            execution_identifier: id.to_string(),
            function_identifier: node.runtime_function_id.clone(),
            parameters: Some(Struct { fields }),
            project_id: 0,
        })
    }
}

fn preview_value(value: &Value) -> String {
    format_value_json(value)
}

fn format_value_json(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::NumberValue(v)) => match v.number {
            Some(kind) => match kind {
                tucana::shared::number_value::Number::Integer(i) => i.to_string(),
                tucana::shared::number_value::Number::Float(f) => f.to_string(),
            },
            _ => "null".to_string(),
        },
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::StringValue(v)) => format!("{:?}", v),
        Some(Kind::NullValue(_)) | None => "null".to_string(),
        Some(Kind::ListValue(list)) => {
            let mut parts = Vec::new();
            for item in list.values.iter() {
                parts.push(format_value_json(item));
            }
            format!("[{}]", parts.join(", "))
        }
        Some(Kind::StructValue(struct_value)) => {
            let mut keys: Vec<_> = struct_value.fields.keys().collect();
            keys.sort();
            let mut parts = Vec::new();
            for key in keys.iter() {
                if let Some(v) = struct_value.fields.get(*key) {
                    parts.push(format!("{:?}: {}", key, format_value_json(v)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn preview_reference(r: &tucana::shared::ReferenceValue) -> String {
    let target = match &r.target {
        Some(Target::FlowInput(_)) => "flow_input".to_string(),
        Some(Target::NodeId(id)) => format!("node({})", id),
        Some(Target::InputType(input_type)) => format!(
            "input(node={},param={},input={})",
            input_type.node_id, input_type.parameter_index, input_type.input_index
        ),
        None => "empty".to_string(),
    };

    if r.paths.is_empty() {
        target
    } else {
        format!("{}+paths({})", target, r.paths.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::registry::{HandlerFn, IntoFunctionEntry};
    use std::cell::RefCell;
    use tucana::shared::number_value::Number;
    use tucana::shared::{NodeFunction, NumberValue, ReferenceValue};

    fn num(i: i64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(NumberValue {
                number: Some(Number::Integer(i)),
            })),
        }
    }

    fn extract_i64(v: &Value) -> i64 {
        match &v.kind {
            Some(Kind::NumberValue(NumberValue {
                number: Some(Number::Integer(i)),
            })) => *i,
            other => panic!("Expected integer number value, got {:?}", other),
        }
    }

    fn node(id: i64, function_id: &str, next: Option<i64>) -> NodeFunction {
        NodeFunction {
            database_id: id,
            runtime_function_id: function_id.to_string(),
            definition_source: "taurus".to_string(),
            next_node_id: next,
            ..Default::default()
        }
    }

    fn node_reference(node_id: i64) -> ReferenceValue {
        ReferenceValue {
            paths: vec![],
            target: Some(Target::NodeId(node_id)),
        }
    }

    fn assert_node_success(ctx: &mut Context, node_id: i64, expected: i64) {
        match ctx.get(node_reference(node_id)) {
            ContextResult::Success(v) => assert_eq!(extract_i64(&v), expected),
            ContextResult::Error(_) => {
                panic!("Expected success result for node {}, got error", node_id)
            }
            ContextResult::NotFound => {
                panic!(
                    "Expected success result for node {}, got not found",
                    node_id
                )
            }
        }
    }

    fn assert_node_error(ctx: &mut Context, node_id: i64, expected_name: &str) {
        match ctx.get(node_reference(node_id)) {
            ContextResult::Error(err) => assert_eq!(err.name, expected_name),
            ContextResult::Success(_) => {
                panic!("Expected error result for node {}, got success", node_id)
            }
            ContextResult::NotFound => {
                panic!("Expected error result for node {}, got not found", node_id)
            }
        }
    }

    fn assert_node_not_found(ctx: &mut Context, node_id: i64) {
        match ctx.get(node_reference(node_id)) {
            ContextResult::NotFound => {}
            ContextResult::Success(_) => {
                panic!("Expected no result for node {}, got success", node_id)
            }
            ContextResult::Error(_) => {
                panic!("Expected no result for node {}, got error", node_id)
            }
        }
    }

    fn success_one_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Success(num(1))
    }

    fn success_two_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Success(num(2))
    }

    fn success_three_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Success(num(3))
    }

    fn failure_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Failure(RuntimeError::simple_str("TestFailure", "expected failure"))
    }

    fn return_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Return(num(9))
    }

    fn stop_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Stop
    }

    fn respond_eleven_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Respond(num(11))
    }

    fn respond_twentytwo_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Respond(num(22))
    }

    fn respond_fortyfour_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Respond(num(44))
    }

    fn return_seventyseven_handler(
        _args: &[Argument],
        _ctx: &mut Context,
        _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        Signal::Return(num(77))
    }

    fn runtime_call_child_two_handler(
        _args: &[Argument],
        ctx: &mut Context,
        run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        match run(2, ctx) {
            Signal::Success(_) => Signal::Success(num(55)),
            other => other,
        }
    }

    fn runtime_call_child_return_handler(
        _args: &[Argument],
        ctx: &mut Context,
        run: &mut dyn FnMut(i64, &mut Context) -> Signal,
    ) -> Signal {
        run(2, ctx)
    }

    #[test]
    fn success_signal_continues_through_next_nodes() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::success_one",
                HandlerFn::eager(success_one_handler, 0),
            ),
            (
                "test::success_two",
                HandlerFn::eager(success_two_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::success_one", Some(2)));
        nodes.insert(2, node(2, "test::success_two", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        match signal {
            Signal::Success(v) => assert_eq!(extract_i64(&v), 2),
            other => panic!("Expected success signal, got {:?}", other),
        }

        assert_node_success(&mut ctx, 1, 1);
        assert_node_success(&mut ctx, 2, 2);
    }

    #[test]
    fn failure_signal_stops_flow_and_persists_error() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::success_one",
                HandlerFn::eager(success_one_handler, 0),
            ),
            ("test::failure", HandlerFn::eager(failure_handler, 0)),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::success_one", Some(2)));
        nodes.insert(2, node(2, "test::failure", Some(3)));
        nodes.insert(3, node(3, "test::success_three", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        match signal {
            Signal::Failure(err) => assert_eq!(err.name, "TestFailure"),
            other => panic!("Expected failure signal, got {:?}", other),
        }

        assert_node_success(&mut ctx, 1, 1);
        assert_node_error(&mut ctx, 2, "TestFailure");
        assert_node_not_found(&mut ctx, 3);
    }

    #[test]
    fn return_signal_breaks_execution_and_skips_next_nodes() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            ("test::return", HandlerFn::eager(return_handler, 0)),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::return", Some(2)));
        nodes.insert(2, node(2, "test::success_three", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        match signal {
            Signal::Return(v) => assert_eq!(extract_i64(&v), 9),
            other => panic!("Expected return signal, got {:?}", other),
        }

        assert_node_not_found(&mut ctx, 1);
        assert_node_not_found(&mut ctx, 2);
    }

    #[test]
    fn stop_signal_stops_execution_and_skips_next_nodes() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            ("test::stop", HandlerFn::eager(stop_handler, 0)),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::stop", Some(2)));
        nodes.insert(2, node(2, "test::success_three", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        assert!(matches!(signal, Signal::Stop));
        assert_node_not_found(&mut ctx, 1);
        assert_node_not_found(&mut ctx, 2);
    }

    #[test]
    fn respond_signal_emits_for_each_node_and_execution_continues() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::respond_eleven",
                HandlerFn::eager(respond_eleven_handler, 0),
            ),
            (
                "test::respond_twentytwo",
                HandlerFn::eager(respond_twentytwo_handler, 0),
            ),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::respond_eleven", Some(2)));
        nodes.insert(2, node(2, "test::respond_twentytwo", Some(3)));
        nodes.insert(3, node(3, "test::success_three", None));

        let emitted = RefCell::new(Vec::<Value>::new());
        let emitter = |v: Value| {
            emitted.borrow_mut().push(v);
        };

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes)
            .with_respond_emitter(&emitter)
            .execute(1, &mut ctx, false);

        match signal {
            Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
            other => panic!("Expected final success signal, got {:?}", other),
        }

        let emitted_values = emitted.borrow();
        assert_eq!(emitted_values.len(), 2);
        assert_eq!(extract_i64(&emitted_values[0]), 11);
        assert_eq!(extract_i64(&emitted_values[1]), 22);

        assert_node_success(&mut ctx, 1, 11);
        assert_node_success(&mut ctx, 2, 22);
        assert_node_success(&mut ctx, 3, 3);
    }

    #[test]
    fn respond_signal_emits_inside_runtime_call_and_parent_continues() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::runtime_call_child_two",
                HandlerFn::eager(runtime_call_child_two_handler, 0),
            ),
            (
                "test::respond_fortyfour",
                HandlerFn::eager(respond_fortyfour_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::runtime_call_child_two", None));
        nodes.insert(2, node(2, "test::respond_fortyfour", None));

        let emitted = RefCell::new(Vec::<Value>::new());
        let emitter = |v: Value| {
            emitted.borrow_mut().push(v);
        };

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes)
            .with_respond_emitter(&emitter)
            .execute(1, &mut ctx, false);

        match signal {
            Signal::Success(v) => assert_eq!(extract_i64(&v), 55),
            other => panic!("Expected final success signal, got {:?}", other),
        }

        let emitted_values = emitted.borrow();
        assert_eq!(emitted_values.len(), 1);
        assert_eq!(extract_i64(&emitted_values[0]), 44);

        assert_node_success(&mut ctx, 1, 55);
        assert_node_success(&mut ctx, 2, 44);
    }

    #[test]
    fn return_signal_in_child_context_continues_parent_flow() {
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::runtime_call_child_return",
                HandlerFn::eager(runtime_call_child_return_handler, 0),
            ),
            (
                "test::return_seventyseven",
                HandlerFn::eager(return_seventyseven_handler, 0),
            ),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::runtime_call_child_return", Some(3)));
        nodes.insert(2, node(2, "test::return_seventyseven", None));
        nodes.insert(3, node(3, "test::success_three", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        match signal {
            Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
            other => panic!("Expected final success signal, got {:?}", other),
        }

        // Child return exits its own context and is treated as value by parent node.
        assert_node_success(&mut ctx, 1, 77);
        assert_node_not_found(&mut ctx, 2);
        assert_node_success(&mut ctx, 3, 3);
    }

    #[test]
    fn child_return_skips_child_next_but_parent_next_runs() {
        // Graph:
        // node1 -> next node4, and node1 executes child node2 via run(...)
        // node2 -> RETURN and has next node3
        // node3 must NOT run (because node2 returned)
        // node4 must run (parent chain continues)
        let mut store = FunctionStore::new();
        store.populate(vec![
            (
                "test::runtime_call_child_return",
                HandlerFn::eager(runtime_call_child_return_handler, 0),
            ),
            (
                "test::return_seventyseven",
                HandlerFn::eager(return_seventyseven_handler, 0),
            ),
            (
                "test::success_two",
                HandlerFn::eager(success_two_handler, 0),
            ),
            (
                "test::success_three",
                HandlerFn::eager(success_three_handler, 0),
            ),
        ]);

        let mut nodes = HashMap::new();
        nodes.insert(1, node(1, "test::runtime_call_child_return", Some(4)));
        nodes.insert(2, node(2, "test::return_seventyseven", Some(3)));
        nodes.insert(3, node(3, "test::success_two", None));
        nodes.insert(4, node(4, "test::success_three", None));

        let mut ctx = Context::default();
        let signal = Executor::new(&store, nodes).execute(1, &mut ctx, false);

        match signal {
            Signal::Success(v) => assert_eq!(extract_i64(&v), 3),
            other => panic!("Expected final success signal, got {:?}", other),
        }

        // node1 ran and received child return value
        assert_node_success(&mut ctx, 1, 77);
        // node2 returned, so it has no success/error entry in context
        assert_node_not_found(&mut ctx, 2);
        // node3 is child next node and must be skipped due to return in node2
        assert_node_not_found(&mut ctx, 3);
        // parent next node still executes
        assert_node_success(&mut ctx, 4, 3);
    }
}
