//! Runtime engine execution loop for compiled flow plans.

use std::cell::RefCell;
use std::collections::HashMap;

use futures_lite::future::block_on;
use tucana::aquila::ExecutionRequest;
use tucana::shared::reference_value::Target;
use tucana::shared::value::Kind;
use tucana::shared::{Struct, Value};
use uuid::Uuid;

use crate::handler::argument::{Argument, ParameterNode};
use crate::handler::registry::{FunctionStore, HandlerFunctionEntry};
use crate::runtime::engine::emitter::{EmitType, RespondEmitter};
use crate::runtime::engine::model::{CompiledArg, CompiledFlow, CompiledNode, NodeExecutionTarget};
use crate::runtime::execution::trace::{
    ArgKind, ArgTrace, EdgeKind, Outcome, ReferenceKind, TraceRun,
};
use crate::runtime::execution::tracer::{ExecutionTracer, Tracer};
use crate::runtime::execution::value_store::{ValueStore, ValueStoreResult};
use crate::runtime::remote::{RemoteExecution, RemoteRuntime};
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;

pub fn execute_compiled(
    flow: &CompiledFlow,
    handlers: &FunctionStore,
    value_store: &mut ValueStore,
    remote: Option<&dyn RemoteRuntime>,
    respond_emitter: Option<&dyn RespondEmitter>,
    with_trace: bool,
) -> (Signal, Option<TraceRun>) {
    // Keep trace allocation fully optional so the hot path stays lean when tracing is disabled.
    let tracer = with_trace.then(RefCell::default);
    let executor = EngineExecutor {
        flow,
        handlers,
        remote,
        respond_emitter,
        tracer: tracer.as_ref(),
    };

    let result = executor.execute_from_index(flow.start_idx, value_store);
    let trace = tracer.and_then(|collector| collector.into_inner().take_run());
    (result.signal, trace)
}

/// Result of executing one linear node chain (entry node + `next` links).
/// `root_frame` is used to connect this chain into the caller frame in trace mode.
struct ExecutionResult {
    signal: Signal,
    root_frame: Option<u64>,
}

/// Result of executing exactly one compiled node.
struct NodeExecutionResult {
    signal: Signal,
    frame_id: Option<u64>,
}

struct EngineExecutor<'a> {
    flow: &'a CompiledFlow,
    handlers: &'a FunctionStore,
    remote: Option<&'a dyn RemoteRuntime>,
    respond_emitter: Option<&'a dyn RespondEmitter>,
    tracer: Option<&'a RefCell<Tracer>>,
}

impl<'a> EngineExecutor<'a> {
    fn execute_from_index(
        &self,
        start_idx: usize,
        value_store: &mut ValueStore,
    ) -> ExecutionResult {
        // A compiled flow is executed as a linear walk through `next_idx` pointers.
        let mut current_idx = start_idx;
        let mut call_root_frame = None;
        let mut previous_frame = None;

        loop {
            let node_id = self.flow.nodes[current_idx].id;
            let next_idx = self.flow.nodes[current_idx].next_idx;
            let result = self.execute_single_node(current_idx, value_store);

            if call_root_frame.is_none() {
                call_root_frame = result.frame_id;
            }
            if let (Some(prev), Some(current)) = (previous_frame, result.frame_id) {
                self.trace_link_child(prev, current, EdgeKind::Next);
            }
            if let Some(frame) = result.frame_id {
                previous_frame = Some(frame);
            }

            match result.signal {
                // Only `Success` keeps walking through the current linear chain.
                Signal::Success(_) => match next_idx {
                    Some(next) => current_idx = next,
                    None => {
                        return ExecutionResult {
                            signal: result.signal,
                            root_frame: call_root_frame,
                        };
                    }
                },
                Signal::Respond(value) => {
                    // `Respond` is an observable side effect; execution may still continue.
                    if let Some(emitter) = self.respond_emitter {
                        emitter.emit(EmitType::OngoingExec, value.clone());
                    }

                    value_store.insert_success(node_id, value.clone());
                    match next_idx {
                        Some(next) => current_idx = next,
                        None => {
                            return ExecutionResult {
                                signal: Signal::Success(value),
                                root_frame: call_root_frame,
                            };
                        }
                    }
                }
                // `Return`/`Stop`/`Failure` unwind immediately to the direct caller.
                other => {
                    return ExecutionResult {
                        signal: other,
                        root_frame: call_root_frame,
                    };
                }
            }
        }
    }

    fn execute_from_node_id(&self, node_id: i64, value_store: &mut ValueStore) -> ExecutionResult {
        // Used by thunk execution (callbacks, branch blocks, eager parameter nodes).
        match self.flow.node_idx_by_id.get(&node_id).copied() {
            Some(idx) => self.execute_from_index(idx, value_store),
            None => ExecutionResult {
                signal: Signal::Failure(RuntimeError::new(
                    "T-ENG-000001",
                    "NodeNotFound",
                    format!("Node {} not found", node_id),
                )),
                root_frame: None,
            },
        }
    }

    fn execute_single_node(
        &self,
        node_idx: usize,
        value_store: &mut ValueStore,
    ) -> NodeExecutionResult {
        let node = &self.flow.nodes[node_idx];
        // InputType references resolve against the currently running node.
        value_store.set_current_node_id(node.id);

        let frame_id = self.trace_enter(node);
        let signal = match &node.execution_target {
            NodeExecutionTarget::Local => self.execute_local_node(node, value_store, frame_id),
            NodeExecutionTarget::Remote { service } => {
                self.execute_remote_node(node, service, value_store, frame_id)
            }
        };
        self.trace_exit(frame_id, &signal);

        NodeExecutionResult { signal, frame_id }
    }

    fn execute_local_node(
        &self,
        node: &CompiledNode,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Signal {
        let entry = match self.handlers.get(node.handler_id.as_str()) {
            Some(entry) => entry,
            None => {
                return Signal::Failure(RuntimeError::new(
                    "T-ENG-000002",
                    "FunctionNotFound",
                    format!("Function {} not found", node.handler_id),
                ));
            }
        };

        let mut args = match self.build_args(node, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                value_store.insert_error(node.id, err.clone());
                return Signal::Failure(err);
            }
        };

        if let Some(signal) = self.force_eager_args(entry, &mut args, value_store, frame_id) {
            return self.commit_result(node.id, signal, value_store);
        }

        // Handler-owned runtime calls (for lazy args / callbacks) re-enter the same executor.
        let mut run = |node_id: i64, store: &mut ValueStore| {
            self.trace_mark_thunk_executed_by_node(frame_id, node_id);
            let label = store.pop_runtime_trace_label();
            let child_result = self.execute_from_node_id(node_id, store);
            if let (Some(parent), Some(child)) = (frame_id, child_result.root_frame) {
                self.trace_link_child(parent, child, EdgeKind::RuntimeCall { label });
            }
            child_result.signal
        };

        let signal = (entry.handler)(&args, value_store, &mut run);
        self.commit_result(node.id, signal, value_store)
    }

    fn execute_remote_node(
        &self,
        node: &CompiledNode,
        service: &str,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Signal {
        let remote_runtime = match self.remote {
            Some(remote) => remote,
            None => {
                return Signal::Failure(RuntimeError::new(
                    "T-ENG-000003",
                    "RemoteRuntimeNotConfigured",
                    "Remote runtime not configured",
                ));
            }
        };

        let mut args = match self.build_args(node, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                value_store.insert_error(node.id, err.clone());
                return Signal::Failure(err);
            }
        };

        let values = match self.resolve_remote_args(&mut args, value_store, frame_id) {
            Ok(values) => values,
            Err(signal) => return self.commit_result(node.id, signal, value_store),
        };

        let request = match self.build_remote_request(node, values) {
            Ok(request) => request,
            Err(err) => {
                value_store.insert_error(node.id, err.clone());
                return Signal::Failure(err);
            }
        };

        let signal = match block_on(remote_runtime.execute_remote(RemoteExecution {
            target_service: service.to_string(),
            request,
        })) {
            Ok(value) => Signal::Success(value),
            Err(err) => Signal::Failure(err),
        };

        self.commit_result(node.id, signal, value_store)
    }

    fn build_args(
        &self,
        node: &CompiledNode,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<Vec<Argument>, RuntimeError> {
        let mut args = Vec::with_capacity(node.parameters.len());

        for (index, parameter) in node.parameters.iter().enumerate() {
            match &parameter.arg {
                CompiledArg::Literal(value) => {
                    self.trace_record_arg(
                        frame_id,
                        ArgTrace {
                            index,
                            kind: ArgKind::Literal,
                            preview: preview_value(value),
                        },
                    );
                    args.push(Argument::Eval(value.clone()));
                }
                CompiledArg::Reference(reference) => match value_store.get(reference.clone()) {
                    ValueStoreResult::Success(value) => {
                        self.trace_record_arg(
                            frame_id,
                            ArgTrace {
                                index,
                                kind: ArgKind::Reference {
                                    reference: match &reference.target {
                                        Some(Target::FlowInput(_)) => ReferenceKind::FlowInput,
                                        Some(Target::NodeId(id)) => {
                                            ReferenceKind::Result { node_id: *id }
                                        }
                                        Some(Target::InputType(input_type)) => {
                                            ReferenceKind::InputType {
                                                node_id: input_type.node_id,
                                                input_index: input_type.input_index,
                                                parameter_index: input_type.parameter_index,
                                            }
                                        }
                                        None => ReferenceKind::Empty,
                                    },
                                    hit: true,
                                },
                                preview: format!(
                                    "store.get({}) -> {}",
                                    preview_reference(reference),
                                    preview_value(&value)
                                ),
                            },
                        );
                        args.push(Argument::Eval(value));
                    }
                    ValueStoreResult::Error(err) => {
                        self.trace_record_arg(
                            frame_id,
                            ArgTrace {
                                index,
                                kind: ArgKind::Reference {
                                    reference: match &reference.target {
                                        Some(Target::FlowInput(_)) => ReferenceKind::FlowInput,
                                        Some(Target::NodeId(id)) => {
                                            ReferenceKind::Result { node_id: *id }
                                        }
                                        Some(Target::InputType(input_type)) => {
                                            ReferenceKind::InputType {
                                                node_id: input_type.node_id,
                                                input_index: input_type.input_index,
                                                parameter_index: input_type.parameter_index,
                                            }
                                        }
                                        None => ReferenceKind::Empty,
                                    },
                                    hit: false,
                                },
                                preview: format!(
                                    "store.get({}) -> error({}:{})",
                                    preview_reference(reference),
                                    err.code,
                                    err.category
                                ),
                            },
                        );
                        return Err(err);
                    }
                    ValueStoreResult::NotFound => {
                        self.trace_record_arg(
                            frame_id,
                            ArgTrace {
                                index,
                                kind: ArgKind::Reference {
                                    reference: match &reference.target {
                                        Some(Target::FlowInput(_)) => ReferenceKind::FlowInput,
                                        Some(Target::NodeId(id)) => {
                                            ReferenceKind::Result { node_id: *id }
                                        }
                                        Some(Target::InputType(input_type)) => {
                                            ReferenceKind::InputType {
                                                node_id: input_type.node_id,
                                                input_index: input_type.input_index,
                                                parameter_index: input_type.parameter_index,
                                            }
                                        }
                                        None => ReferenceKind::Empty,
                                    },
                                    hit: false,
                                },
                                preview: format!(
                                    "store.get({}) -> not-found",
                                    preview_reference(reference)
                                ),
                            },
                        );
                        return Err(RuntimeError::new(
                            "T-ENG-000004",
                            "ReferenceValueNotFound",
                            "Reference not found in execution value store",
                        ));
                    }
                },
                CompiledArg::DeferredNode(node_id) => {
                    self.trace_record_arg(
                        frame_id,
                        ArgTrace {
                            index,
                            kind: ArgKind::Thunk {
                                node_id: *node_id,
                                eager: false,
                                executed: false,
                            },
                            preview: format!("thunk({})", node_id),
                        },
                    );
                    args.push(Argument::Thunk(*node_id));
                }
            }
        }

        Ok(args)
    }

    fn force_eager_args(
        &self,
        entry: &HandlerFunctionEntry,
        args: &mut [Argument],
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Option<Signal> {
        for (index, argument) in args.iter_mut().enumerate() {
            let mode = entry.param_mode(index);

            if matches!(mode, ParameterNode::Eager)
                && let Argument::Thunk(node_id) = *argument
            {
                self.trace_mark_thunk(frame_id, index, true, true);
                let child = self.execute_from_node_id(node_id, value_store);
                if let (Some(parent), Some(child_root)) = (frame_id, child.root_frame) {
                    self.trace_link_child(
                        parent,
                        child_root,
                        EdgeKind::EagerCall { arg_index: index },
                    );
                }
                match child.signal {
                    Signal::Success(value) => {
                        *argument = Argument::Eval(value);
                    }
                    // Return in an eager parameter block exits only this node invocation,
                    // so the caller continues with its own `next` node.
                    Signal::Return(value) => return Some(Signal::Success(value)),
                    other => return Some(other),
                }
            }
        }

        None
    }

    fn resolve_remote_args(
        &self,
        args: &mut [Argument],
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<Vec<Value>, Signal> {
        let mut values = Vec::with_capacity(args.len());

        for (index, argument) in args.iter_mut().enumerate() {
            match argument {
                Argument::Eval(value) => values.push(value.clone()),
                Argument::Thunk(node_id) => {
                    // Remote execution always receives materialized values, never thunks.
                    self.trace_mark_thunk(frame_id, index, true, true);
                    let child = self.execute_from_node_id(*node_id, value_store);
                    if let (Some(parent), Some(child_root)) = (frame_id, child.root_frame) {
                        self.trace_link_child(
                            parent,
                            child_root,
                            EdgeKind::EagerCall { arg_index: index },
                        );
                    }
                    match child.signal {
                        Signal::Success(value) => {
                            *argument = Argument::Eval(value.clone());
                            values.push(value);
                        }
                        // Same unwind rule as local eager params: return exits this call frame only.
                        Signal::Return(value) => return Err(Signal::Success(value)),
                        other => return Err(other),
                    }
                }
            }
        }

        Ok(values)
    }

    fn build_remote_request(
        &self,
        node: &CompiledNode,
        values: Vec<Value>,
    ) -> Result<ExecutionRequest, RuntimeError> {
        if node.parameters.len() != values.len() {
            return Err(RuntimeError::new(
                "T-ENG-000005",
                "RemoteParameterMismatch",
                "Remote parameter count mismatch",
            ));
        }

        let mut fields = HashMap::new();
        for (parameter, value) in node.parameters.iter().zip(values.into_iter()) {
            fields.insert(parameter.runtime_parameter_id.clone(), value);
        }

        Ok(ExecutionRequest {
            execution_identifier: Uuid::new_v4().to_string(),
            function_identifier: node.handler_id.clone(),
            parameters: Some(Struct { fields }),
            project_id: 0,
        })
    }

    fn commit_result(&self, node_id: i64, signal: Signal, value_store: &mut ValueStore) -> Signal {
        match signal {
            Signal::Success(value) => {
                value_store.insert_success(node_id, value.clone());
                Signal::Success(value)
            }
            Signal::Failure(err) => {
                value_store.insert_error(node_id, err.clone());
                Signal::Failure(err)
            }
            // Control signals are transient and should not be cached as node outputs.
            other => other,
        }
    }

    fn trace_enter(&self, node: &CompiledNode) -> Option<u64> {
        self.tracer.map(|tracer| {
            tracer
                .borrow_mut()
                .enter_node(node.id, node.handler_id.as_str())
        })
    }

    fn trace_exit(&self, frame_id: Option<u64>, signal: &Signal) {
        let Some(frame_id) = frame_id else {
            return;
        };
        let Some(tracer) = self.tracer else {
            return;
        };

        let outcome = match signal {
            Signal::Success(value) => Outcome::Success {
                value_preview: preview_value(value),
            },
            Signal::Failure(error) => Outcome::Failure {
                error_preview: format!("{}:{} {}", error.code, error.category, error.message),
            },
            Signal::Return(value) => Outcome::Return {
                value_preview: preview_value(value),
            },
            Signal::Respond(value) => Outcome::Respond {
                value_preview: preview_value(value),
            },
            Signal::Stop => Outcome::Stop,
        };
        tracer.borrow_mut().exit_node(frame_id, outcome);
    }

    fn trace_record_arg(&self, frame_id: Option<u64>, arg: ArgTrace) {
        if let (Some(frame_id), Some(tracer)) = (frame_id, self.tracer) {
            tracer.borrow_mut().record_arg(frame_id, arg);
        }
    }

    fn trace_link_child(&self, parent: u64, child: u64, edge: EdgeKind) {
        if let Some(tracer) = self.tracer {
            tracer.borrow_mut().link_child(parent, child, edge);
        }
    }

    fn trace_mark_thunk(
        &self,
        frame_id: Option<u64>,
        arg_index: usize,
        eager: bool,
        executed: bool,
    ) {
        if let (Some(frame_id), Some(tracer)) = (frame_id, self.tracer) {
            tracer
                .borrow_mut()
                .mark_thunk(frame_id, arg_index, eager, executed);
        }
    }

    fn trace_mark_thunk_executed_by_node(&self, frame_id: Option<u64>, node_id: i64) {
        if let (Some(frame_id), Some(tracer)) = (frame_id, self.tracer) {
            tracer
                .borrow_mut()
                .mark_thunk_executed_by_node(frame_id, node_id);
        }
    }
}

fn preview_value(value: &Value) -> String {
    // Trace previews are deterministic and human-readable for debugging snapshots.
    format_value_json(value)
}

fn format_value_json(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::NumberValue(v)) => crate::value::number_to_string(v),
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::StringValue(v)) => format!("{:?}", v),
        Some(Kind::NullValue(_)) | None => "null".to_string(),
        Some(Kind::ListValue(list)) => {
            let mut parts = Vec::new();
            for item in &list.values {
                parts.push(format_value_json(item));
            }
            format!("[{}]", parts.join(", "))
        }
        Some(Kind::StructValue(struct_value)) => {
            let mut keys: Vec<_> = struct_value.fields.keys().collect();
            keys.sort();
            let mut parts = Vec::new();
            for key in &keys {
                if let Some(value) = struct_value.fields.get(*key) {
                    parts.push(format!("{:?}: {}", key, format_value_json(value)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn preview_reference(reference: &tucana::shared::ReferenceValue) -> String {
    let target = match &reference.target {
        Some(Target::FlowInput(_)) => "flow_input".to_string(),
        Some(Target::NodeId(id)) => format!("node({})", id),
        Some(Target::InputType(input_type)) => format!(
            "input(node={},param={},input={})",
            input_type.node_id, input_type.parameter_index, input_type.input_index
        ),
        None => "empty".to_string(),
    };

    if reference.paths.is_empty() {
        target
    } else {
        format!("{}+paths({})", target, reference.paths.len())
    }
}
