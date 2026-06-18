//! Runtime engine execution loop for compiled flow plans.

use std::collections::HashMap;
use std::sync::Mutex;

use tucana::aquila::ActionExecutionRequest;
use tucana::shared::node_execution_result::Result as TucanaNodeResult;
use tucana::shared::reference_value::Target;
use tucana::shared::value::Kind;
use tucana::shared::{
    InputType, NodeExecutionResult as TucanaNodeExecutionResult, NodeParameterNodeExecutionResult,
    ReferenceValue, Struct, SubFlowSetting, Value,
};
use uuid::Uuid;

use crate::handler::argument::{Argument, FunctionThunk, ParameterNode, Thunk};
use crate::handler::registry::{FunctionStore, HandlerFunctionEntry};
use crate::runtime::engine::emitter::{EmitType, ExecutionId, RespondEmitter};
use crate::runtime::engine::model::{
    CompiledArg, CompiledFlow, CompiledNode, CompiledThunk, NodeExecutionTarget,
};
use crate::runtime::execution::trace::{
    ArgKind, ArgTrace, EdgeKind, Outcome, ReferenceKind, TraceRun,
};
use crate::runtime::execution::tracer::{ExecutionTracer, Tracer};
use crate::runtime::execution::value_store::{ValueStore, ValueStoreResult};
use crate::runtime::remote::{RemoteExecution, RemoteRuntime};
use crate::time::now_unix_micros;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;

pub async fn execute_compiled(
    flow: &CompiledFlow,
    handlers: &FunctionStore,
    value_store: &mut ValueStore,
    remote: Option<&dyn RemoteRuntime>,
    execution_id: ExecutionId,
    respond_emitter: Option<&dyn RespondEmitter>,
    with_trace: bool,
) -> (Signal, Option<TraceRun>) {
    // Keep trace allocation fully optional so the hot path stays lean when tracing is disabled.
    let tracer = with_trace.then(Mutex::default);
    let executor = EngineExecutor {
        flow,
        handlers,
        remote,
        execution_id,
        respond_emitter,
        tracer: tracer.as_ref(),
    };

    let result = executor
        .execute_from_index(flow.start_idx, value_store)
        .await;
    let trace = tracer.and_then(|collector| collector.into_inner().ok()?.take_run());
    (result.signal, trace)
}

/// Result of executing one linear node chain (entry node + `next` links).
/// `root_frame` is used to connect this chain into the caller frame in trace mode.
struct ExecutionResult {
    signal: Signal,
    root_frame: Option<u64>,
}

/// Result of executing exactly one compiled node.
struct NodeResult {
    signal: Signal,
    frame_id: Option<u64>,
    parameter_results: Vec<NodeParameterNodeExecutionResult>,
    started_at: i64,
    finished_at: i64,
}

struct ExecutedNode {
    signal: Signal,
    parameter_results: Vec<NodeParameterNodeExecutionResult>,
}

struct EngineExecutor<'a> {
    flow: &'a CompiledFlow,
    handlers: &'a FunctionStore,
    remote: Option<&'a dyn RemoteRuntime>,
    execution_id: ExecutionId,
    respond_emitter: Option<&'a dyn RespondEmitter>,
    tracer: Option<&'a Mutex<Tracer>>,
}

impl<'a> EngineExecutor<'a> {
    async fn execute_from_index(
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
            let result = self.execute_single_node(current_idx, value_store).await;

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
                Signal::Success(value) => match next_idx {
                    Some(next) => current_idx = next,
                    None => {
                        return ExecutionResult {
                            signal: Signal::Success(value),
                            root_frame: call_root_frame,
                        };
                    }
                },
                Signal::Respond(value) => {
                    // `Respond` is an observable side effect; execution may still continue.
                    if let Some(emitter) = self.respond_emitter {
                        emitter.emit(self.execution_id, EmitType::OngoingExec, value.clone());
                    }

                    value_store.insert_success_with_timing(
                        node_id,
                        value.clone(),
                        result.parameter_results,
                        result.started_at,
                        result.finished_at,
                    );
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

    fn execute_from_index_sync(
        &self,
        start_idx: usize,
        value_store: &mut ValueStore,
    ) -> ExecutionResult {
        // Synchronous thunk execution is retained for local handler callbacks.
        let mut current_idx = start_idx;
        let mut call_root_frame = None;
        let mut previous_frame = None;

        loop {
            let node_id = self.flow.nodes[current_idx].id;
            let next_idx = self.flow.nodes[current_idx].next_idx;
            let result = self.execute_single_node_sync(current_idx, value_store);

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
                Signal::Success(value) => match next_idx {
                    Some(next) => current_idx = next,
                    None => {
                        return ExecutionResult {
                            signal: Signal::Success(value),
                            root_frame: call_root_frame,
                        };
                    }
                },
                Signal::Respond(value) => {
                    if let Some(emitter) = self.respond_emitter {
                        emitter.emit(self.execution_id, EmitType::OngoingExec, value.clone());
                    }

                    value_store.insert_success_with_timing(
                        node_id,
                        value.clone(),
                        result.parameter_results,
                        result.started_at,
                        result.finished_at,
                    );
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
            Some(idx) => self.execute_from_index_sync(idx, value_store),
            None => ExecutionResult {
                signal: Signal::Failure(RuntimeError::new(
                    "T-CORE-000001",
                    "NodeNotFound",
                    format!("Node {} not found", node_id),
                )),
                root_frame: None,
            },
        }
    }

    fn execute_thunk(&self, thunk: &Thunk, value_store: &mut ValueStore) -> ExecutionResult {
        match thunk {
            Thunk::Node(node_id) => self.execute_from_node_id(*node_id, value_store),
            Thunk::Function(function) => self.execute_function_thunk(function, value_store),
        }
    }

    fn execute_function_thunk(
        &self,
        function: &FunctionThunk,
        value_store: &mut ValueStore,
    ) -> ExecutionResult {
        let started_at = now_unix_micros();
        let entry = match self.handlers.get(function.identifier.as_str()).copied() {
            Some(entry) => entry,
            None => {
                let error = RuntimeError::new(
                    "T-CORE-000002",
                    "FunctionNotFound",
                    format!("Function {} not found", function.identifier),
                );
                value_store.insert_function_error_with_timing(
                    function.identifier.clone(),
                    error.clone(),
                    Vec::new(),
                    started_at,
                    now_unix_micros(),
                );
                return ExecutionResult {
                    signal: Signal::Failure(error),
                    root_frame: None,
                };
            }
        };

        let frame_id = self.trace_enter_function(
            value_store.get_current_node_id(),
            function.identifier.as_str(),
            value_store,
        );

        let mut args = match self.build_function_thunk_args(function, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                let signal = Signal::Failure(err);
                self.trace_exit(frame_id, &signal, value_store);
                let parameter_results = Vec::new();
                self.commit_function_result(
                    function.identifier.as_str(),
                    signal.clone(),
                    parameter_results,
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
                return ExecutionResult {
                    signal,
                    root_frame: frame_id,
                };
            }
        };
        let parameter_results = parameter_results_from_args(&args);

        let signal =
            if let Some(signal) = self.force_eager_args(&entry, &mut args, value_store, frame_id) {
                signal
            } else {
                let mut run = |thunk: &Thunk, store: &mut ValueStore| {
                    self.trace_mark_thunk_executed(frame_id, thunk);
                    let label = store.pop_runtime_trace_label();
                    let child_result = self.execute_thunk(thunk, store);
                    if let (Some(parent), Some(child)) = (frame_id, child_result.root_frame) {
                        self.trace_link_child(parent, child, EdgeKind::RuntimeCall { label });
                    }
                    child_result.signal
                };

                (entry.handler)(&args, value_store, &mut run)
            };

        self.trace_exit(frame_id, &signal, value_store);
        self.commit_function_result(
            function.identifier.as_str(),
            signal.clone(),
            parameter_results,
            started_at,
            now_unix_micros(),
            value_store,
        );

        ExecutionResult {
            signal,
            root_frame: frame_id,
        }
    }

    async fn execute_single_node(
        &self,
        node_idx: usize,
        value_store: &mut ValueStore,
    ) -> NodeResult {
        let node = &self.flow.nodes[node_idx];
        // InputType references resolve against the currently running node.
        value_store.set_current_node_id(node.id);

        let frame_id = self.trace_enter(node, value_store);
        let result = match &node.execution_target {
            NodeExecutionTarget::Local => {
                let started_at = now_unix_micros();
                let executed = self.execute_local_node(node, value_store, frame_id);
                let finished_at = now_unix_micros();
                let parameter_results = executed.parameter_results;
                let signal = self.commit_result(
                    node.id,
                    executed.signal,
                    parameter_results.clone(),
                    started_at,
                    finished_at,
                    value_store,
                );
                NodeResult {
                    signal,
                    frame_id,
                    parameter_results,
                    started_at,
                    finished_at,
                }
            }
            NodeExecutionTarget::Remote { service } => {
                let started_at = now_unix_micros();
                let signal = self
                    .execute_remote_node(node, service, value_store, frame_id)
                    .await;
                let finished_at = now_unix_micros();
                NodeResult {
                    signal,
                    frame_id,
                    parameter_results: Vec::new(),
                    started_at,
                    finished_at,
                }
            }
        };
        self.trace_exit(frame_id, &result.signal, value_store);

        result
    }

    fn execute_single_node_sync(
        &self,
        node_idx: usize,
        value_store: &mut ValueStore,
    ) -> NodeResult {
        let node = &self.flow.nodes[node_idx];
        value_store.set_current_node_id(node.id);

        let frame_id = self.trace_enter(node, value_store);
        let result = match &node.execution_target {
            NodeExecutionTarget::Local => {
                let started_at = now_unix_micros();
                let executed = self.execute_local_node(node, value_store, frame_id);
                let finished_at = now_unix_micros();
                let parameter_results = executed.parameter_results;
                let signal = self.commit_result(
                    node.id,
                    executed.signal,
                    parameter_results.clone(),
                    started_at,
                    finished_at,
                    value_store,
                );
                NodeResult {
                    signal,
                    frame_id,
                    parameter_results,
                    started_at,
                    finished_at,
                }
            }
            NodeExecutionTarget::Remote { .. } => {
                let started_at = now_unix_micros();
                let signal = self.commit_result(
                    node.id,
                    Signal::Failure(RuntimeError::new(
                        "T-CORE-000004",
                        "RemoteRuntimeRequiresAsyncExecution",
                        "Remote runtime nodes cannot be executed from a synchronous thunk callback",
                    )),
                    Vec::new(),
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
                NodeResult {
                    signal,
                    frame_id,
                    parameter_results: Vec::new(),
                    started_at,
                    finished_at: now_unix_micros(),
                }
            }
        };
        self.trace_exit(frame_id, &result.signal, value_store);

        result
    }

    fn execute_local_node(
        &self,
        node: &CompiledNode,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> ExecutedNode {
        let entry = match self.handlers.get(node.handler_id.as_str()) {
            Some(entry) => entry,
            None => {
                return ExecutedNode {
                    signal: Signal::Failure(RuntimeError::new(
                        "T-CORE-000002",
                        "FunctionNotFound",
                        format!("Function {} not found", node.handler_id),
                    )),
                    parameter_results: Vec::new(),
                };
            }
        };

        let mut args = match self.build_args(node, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                return ExecutedNode {
                    signal: Signal::Failure(err),
                    parameter_results: Vec::new(),
                };
            }
        };

        if let Some(signal) = self.force_eager_args(entry, &mut args, value_store, frame_id) {
            return ExecutedNode {
                signal,
                parameter_results: parameter_results_from_args(&args),
            };
        }

        let parameter_results = parameter_results_from_args(&args);

        // Handler-owned runtime calls (for lazy args / callbacks) re-enter the same executor.
        let mut run = |thunk: &Thunk, store: &mut ValueStore| {
            self.trace_mark_thunk_executed(frame_id, thunk);
            let label = store.pop_runtime_trace_label();
            let child_result = self.execute_thunk(thunk, store);
            if let (Some(parent), Some(child)) = (frame_id, child_result.root_frame) {
                self.trace_link_child(parent, child, EdgeKind::RuntimeCall { label });
            }
            child_result.signal
        };

        ExecutedNode {
            signal: (entry.handler)(&args, value_store, &mut run),
            parameter_results,
        }
    }

    async fn execute_remote_node(
        &self,
        node: &CompiledNode,
        service: &str,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Signal {
        let started_at = now_unix_micros();
        let remote_runtime = match self.remote {
            Some(remote) => remote,
            None => {
                return self.commit_result(
                    node.id,
                    Signal::Failure(RuntimeError::new(
                        "T-CORE-000003",
                        "RemoteRuntimeNotConfigured",
                        "Remote runtime not configured",
                    )),
                    Vec::new(),
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
            }
        };

        let mut args = match self.build_args(node, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                return self.commit_result(
                    node.id,
                    Signal::Failure(err),
                    Vec::new(),
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
            }
        };

        let values = match self.resolve_remote_args(&mut args, value_store, frame_id) {
            Ok(values) => values,
            Err(signal) => {
                return self.commit_result(
                    node.id,
                    signal,
                    parameter_results_from_args(&args),
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
            }
        };
        let parameter_results = parameter_results_from_values(&values);

        let request = match self.build_remote_request(node, values) {
            Ok(request) => request,
            Err(err) => {
                return self.commit_result(
                    node.id,
                    Signal::Failure(err),
                    parameter_results,
                    started_at,
                    now_unix_micros(),
                    value_store,
                );
            }
        };

        match remote_runtime
            .execute_remote(RemoteExecution {
                target_service: service.to_string(),
                request,
            })
            .await
        {
            Ok(result) => self.commit_remote_result(
                node.id,
                result,
                parameter_results,
                started_at,
                now_unix_micros(),
                value_store,
            ),
            Err(err) => self.commit_result(
                node.id,
                Signal::Failure(err),
                parameter_results,
                started_at,
                now_unix_micros(),
                value_store,
            ),
        }
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
                            "T-CORE-000004",
                            "ReferenceValueNotFound",
                            "Reference not found in execution value store",
                        ));
                    }
                },
                CompiledArg::Deferred(thunk) => {
                    let thunk = compiled_thunk_to_argument(thunk);
                    let target = thunk.trace_target();
                    self.trace_record_arg(
                        frame_id,
                        ArgTrace {
                            index,
                            kind: ArgKind::Thunk {
                                target: target.clone(),
                                eager: false,
                                executed: false,
                            },
                            preview: format!("thunk({})", target),
                        },
                    );
                    args.push(Argument::Thunk(thunk));
                }
            }
        }

        Ok(args)
    }

    fn build_function_thunk_args(
        &self,
        function: &FunctionThunk,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<Vec<Argument>, RuntimeError> {
        let mut args = Vec::with_capacity(function.settings.len());
        let current_node_id = value_store.get_current_node_id();

        for (index, setting) in function.settings.iter().enumerate() {
            let input_type = InputType {
                node_id: current_node_id,
                parameter_index: function.parameter_index,
                input_index: index as i64,
            };
            let value = resolve_function_setting(function, setting, input_type, value_store)?;
            self.trace_record_arg(
                frame_id,
                ArgTrace {
                    index,
                    kind: ArgKind::Literal,
                    preview: format!(
                        "setting({}) -> {}",
                        setting.identifier,
                        preview_value(&value)
                    ),
                },
            );
            args.push(Argument::Eval(value));
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
                && let Argument::Thunk(thunk) = argument
            {
                self.trace_mark_thunk(frame_id, index, true, true);
                let child = self.execute_thunk(thunk, value_store);
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
                Argument::Thunk(thunk) => {
                    // Remote execution always receives materialized values, never thunks.
                    self.trace_mark_thunk(frame_id, index, true, true);
                    let child = self.execute_thunk(thunk, value_store);
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
    ) -> Result<ActionExecutionRequest, RuntimeError> {
        if node.parameters.len() != values.len() {
            return Err(RuntimeError::new(
                "T-CORE-000005",
                "RemoteParameterMismatch",
                "Remote parameter count mismatch",
            ));
        }

        let mut fields = HashMap::new();
        for (parameter, value) in node.parameters.iter().zip(values) {
            fields.insert(parameter.runtime_parameter_id.clone(), value);
        }

        Ok(ActionExecutionRequest {
            execution_identifier: Uuid::new_v4().to_string(),
            function_identifier: node.handler_id.clone(),
            parameters: Some(Struct { fields }),
            project_id: 0,
        })
    }

    fn commit_result(
        &self,
        node_id: i64,
        signal: Signal,
        parameter_results: Vec<NodeParameterNodeExecutionResult>,
        started_at: i64,
        finished_at: i64,
        value_store: &mut ValueStore,
    ) -> Signal {
        match signal {
            Signal::Success(value) => {
                value_store.insert_success_with_timing(
                    node_id,
                    value.clone(),
                    parameter_results,
                    started_at,
                    finished_at,
                );
                Signal::Success(value)
            }
            Signal::Failure(err) => {
                value_store.insert_error_with_timing(
                    node_id,
                    err.clone(),
                    parameter_results,
                    started_at,
                    finished_at,
                );
                Signal::Failure(err)
            }
            // Control signals are transient and should not be cached as node outputs.
            other => other,
        }
    }

    fn commit_function_result(
        &self,
        function_id: &str,
        signal: Signal,
        parameter_results: Vec<NodeParameterNodeExecutionResult>,
        started_at: i64,
        finished_at: i64,
        value_store: &mut ValueStore,
    ) -> Signal {
        match signal {
            Signal::Success(value) => {
                value_store.insert_function_success_with_timing(
                    function_id.to_string(),
                    value.clone(),
                    parameter_results,
                    started_at,
                    finished_at,
                );
                Signal::Success(value)
            }
            Signal::Failure(err) => {
                value_store.insert_function_error_with_timing(
                    function_id.to_string(),
                    err.clone(),
                    parameter_results,
                    started_at,
                    finished_at,
                );
                Signal::Failure(err)
            }
            other => other,
        }
    }

    fn commit_remote_result(
        &self,
        node_id: i64,
        mut result: TucanaNodeExecutionResult,
        parameter_results: Vec<NodeParameterNodeExecutionResult>,
        started_at: i64,
        finished_at: i64,
        value_store: &mut ValueStore,
    ) -> Signal {
        if result.parameter_results.is_empty() {
            result.parameter_results = parameter_results;
        }
        match result.result.clone() {
            Some(TucanaNodeResult::Success(value)) => {
                value_store.insert_node_result(node_id, result);
                Signal::Success(value)
            }
            Some(TucanaNodeResult::Error(error)) => {
                value_store.insert_node_result(node_id, result);
                Signal::Failure(RuntimeError::from_tucana_error(&error))
            }
            None => {
                let runtime_error = RuntimeError::new(
                    "T-CORE-000006",
                    "NodeExecutionResultMissingOutcome",
                    "Remote node execution result is missing success/error outcome",
                );
                value_store.insert_error_with_timing(
                    node_id,
                    runtime_error.clone(),
                    result.parameter_results,
                    started_at,
                    finished_at,
                );
                Signal::Failure(runtime_error)
            }
        }
    }

    fn trace_enter(&self, node: &CompiledNode, value_store: &ValueStore) -> Option<u64> {
        self.trace_enter_function(node.id, node.handler_id.as_str(), value_store)
    }

    fn trace_enter_function(
        &self,
        node_id: i64,
        function_name: &str,
        value_store: &ValueStore,
    ) -> Option<u64> {
        self.tracer.map(|tracer| {
            tracer
                .lock()
                .expect("trace collector should not be poisoned")
                .enter_node(node_id, function_name, value_store.trace_snapshot())
        })
    }

    fn trace_exit(&self, frame_id: Option<u64>, signal: &Signal, value_store: &ValueStore) {
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
        tracer
            .lock()
            .expect("trace collector should not be poisoned")
            .exit_node(frame_id, outcome, value_store.trace_snapshot());
    }

    fn trace_record_arg(&self, frame_id: Option<u64>, arg: ArgTrace) {
        if let (Some(frame_id), Some(tracer)) = (frame_id, self.tracer) {
            tracer
                .lock()
                .expect("trace collector should not be poisoned")
                .record_arg(frame_id, arg);
        }
    }

    fn trace_link_child(&self, parent: u64, child: u64, edge: EdgeKind) {
        if let Some(tracer) = self.tracer {
            tracer
                .lock()
                .expect("trace collector should not be poisoned")
                .link_child(parent, child, edge);
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
                .lock()
                .expect("trace collector should not be poisoned")
                .mark_thunk(frame_id, arg_index, eager, executed);
        }
    }

    fn trace_mark_thunk_executed(&self, frame_id: Option<u64>, thunk: &Thunk) {
        if let (Some(frame_id), Some(tracer)) = (frame_id, self.tracer) {
            tracer
                .lock()
                .expect("trace collector should not be poisoned")
                .mark_thunk_executed(frame_id, thunk.trace_target().as_str());
        }
    }
}

fn parameter_results_from_args(args: &[Argument]) -> Vec<NodeParameterNodeExecutionResult> {
    args.iter()
        .map(|arg| NodeParameterNodeExecutionResult {
            value: match arg {
                Argument::Eval(value) => Some(value.clone()),
                Argument::Thunk(_) => None,
            },
        })
        .collect()
}

fn parameter_results_from_values(values: &[Value]) -> Vec<NodeParameterNodeExecutionResult> {
    values
        .iter()
        .map(|value| NodeParameterNodeExecutionResult {
            value: Some(value.clone()),
        })
        .collect()
}

fn compiled_thunk_to_argument(thunk: &CompiledThunk) -> Thunk {
    match thunk {
        CompiledThunk::Node(node_id) => Thunk::Node(*node_id),
        CompiledThunk::Function {
            identifier,
            parameter_index,
            settings,
        } => Thunk::Function(FunctionThunk {
            identifier: identifier.clone(),
            parameter_index: *parameter_index,
            settings: settings.clone(),
        }),
    }
}

fn resolve_function_setting(
    function: &FunctionThunk,
    setting: &SubFlowSetting,
    input_type: InputType,
    value_store: &mut ValueStore,
) -> Result<Value, RuntimeError> {
    if setting.hidden.unwrap_or(false) {
        return Ok(setting_default_or_null(setting));
    }

    let reference = ReferenceValue {
        target: Some(Target::InputType(input_type)),
        paths: Vec::new(),
    };

    match value_store.get(reference) {
        ValueStoreResult::Success(value) => {
            if is_null_value(&value)
                && let Some(default_value) = setting.default_value.clone()
            {
                Ok(default_value)
            } else {
                Ok(value)
            }
        }
        ValueStoreResult::Error(err) => Err(err),
        ValueStoreResult::NotFound => {
            if let Some(default_value) = setting.default_value.clone() {
                Ok(default_value)
            } else if setting.optional.unwrap_or(false) {
                Ok(null_value())
            } else {
                Err(RuntimeError::new(
                    "T-CORE-000107",
                    "SubFlowSettingValueMissing",
                    format!(
                        "Required sub_flow setting {} for function {} is missing",
                        setting.identifier, function.identifier
                    ),
                ))
            }
        }
    }
}

fn setting_default_or_null(setting: &SubFlowSetting) -> Value {
    setting.default_value.clone().unwrap_or_else(null_value)
}

fn is_null_value(value: &Value) -> bool {
    matches!(value.kind.as_ref(), None | Some(Kind::NullValue(_)))
}

fn null_value() -> Value {
    Value {
        kind: Some(Kind::NullValue(0)),
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
