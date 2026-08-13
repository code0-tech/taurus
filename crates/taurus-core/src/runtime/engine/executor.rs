//! Runtime engine execution loop for compiled flow plans.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::handler::argument::{
    Argument, FunctionThunk, ParameterNode, TemplateArgument, TemplateReferenceArgument, Thunk,
};
use crate::handler::registry::{FunctionStore, HandlerFunctionEntry};
use crate::runtime::engine::model::{
    CompiledArg, CompiledFlow, CompiledNode, CompiledThunk, NodeExecutionTarget,
};
use crate::runtime::engine::sub_flow_registry::SubFlowRegistry;
use crate::runtime::execution::trace::{
    ArgKind, ArgTrace, EdgeKind, Outcome, ReferenceKind, TraceRun,
};
use crate::runtime::execution::tracer::{ExecutionTracer, Tracer};
use crate::runtime::execution::value_store::{ValueStore, ValueStoreResult};
use crate::runtime::remote::{RemoteExecution, RemoteRuntime};
use crate::time::now_unix_micros;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use futures_lite::future::block_on;
use tokio::sync::Notify;
use tucana::aquila::{
    ActionExecutionRequest, ActionInlineReferenceValue, ActionLiteralValue, ActionNodeSubFlowValue,
    ActionNodeValue, action_node_value,
};
use tucana::shared::node_execution_result::Result as TucanaNodeResult;
use tucana::shared::reference_value::Target;
use tucana::shared::value::Kind;
use tucana::shared::{
    InputType, ListValue, NodeExecutionResult as TucanaNodeExecutionResult,
    NodeParameterNodeExecutionResult, ReferenceValue, Struct, SubFlowSetting, Value,
};

/// Executes a compiled flow plan starting at `start_idx` -- used both by a
/// normal top-level run (`start_idx == flow.start_idx`) and by the
/// `sub_flow_execution.*` subscriber running a previously minted sub-flow
/// node range standalone (`ExecutionEngine::execute_sub_flow`).
#[allow(clippy::too_many_arguments)]
pub async fn execute_compiled_from(
    execution_id: &str,
    flow: &Arc<CompiledFlow>,
    start_idx: usize,
    handlers: &FunctionStore,
    value_store: &mut ValueStore,
    remote: Option<&dyn RemoteRuntime>,
    with_trace: bool,
    sub_flow_registry: SubFlowRegistry,
) -> (Signal, Option<TraceRun>) {
    // Keep trace allocation fully optional so the hot path stays lean when tracing is disabled.
    let tracer = with_trace.then(Mutex::default);
    let executor = EngineExecutor {
        execution_id,
        flow: Arc::clone(flow),
        handlers,
        remote,
        tracer: tracer.as_ref(),
        sub_flow_registry,
    };

    let result = executor.execute_from_index(start_idx, value_store).await;
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
}

struct ExecutedNode {
    signal: Signal,
    parameter_results: Vec<NodeParameterNodeExecutionResult>,
}

struct EngineExecutor<'a> {
    /// The originating flow execution's id — reused verbatim as the
    /// `execution_identifier` on every remote call this flow makes, so an
    /// action can correlate a callback (e.g. `respond`) against the run
    /// that triggered it.
    execution_id: &'a str,
    /// `Arc`-wrapped so minting a sub-flow registry entry (which captures
    /// the flow standalone -- see `sub_flow_registry`) is a cheap refcount
    /// bump rather than a deep clone of the node graph.
    flow: Arc<CompiledFlow>,
    handlers: &'a FunctionStore,
    remote: Option<&'a dyn RemoteRuntime>,
    tracer: Option<&'a Mutex<Tracer>>,
    sub_flow_registry: SubFlowRegistry,
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
            Thunk::Node { node_id, .. } => self.execute_from_node_id(*node_id, value_store),
            Thunk::Function(function) => self.execute_function_thunk(function, value_store),
        }
    }

    fn execute_function_thunk(
        &self,
        function: &FunctionThunk,
        value_store: &mut ValueStore,
    ) -> ExecutionResult {
        match &function.execution_target {
            NodeExecutionTarget::Local => self.execute_local_function_thunk(function, value_store),
            NodeExecutionTarget::Remote { service } => {
                self.execute_remote_function_thunk(function, service.as_str(), value_store)
            }
        }
    }

    fn execute_local_function_thunk(
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

    fn execute_remote_function_thunk(
        &self,
        function: &FunctionThunk,
        service: &str,
        value_store: &mut ValueStore,
    ) -> ExecutionResult {
        let started_at = now_unix_micros();
        let frame_id = self.trace_enter_function(
            value_store.get_current_node_id(),
            function.identifier.as_str(),
            value_store,
        );

        let args = match self.build_function_thunk_args(function, value_store, frame_id) {
            Ok(args) => args,
            Err(err) => {
                let signal = Signal::Failure(err);
                self.trace_exit(frame_id, &signal, value_store);
                self.commit_function_result(
                    function.identifier.as_str(),
                    signal.clone(),
                    Vec::new(),
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

        let signal = match self.remote {
            None => Signal::Failure(RuntimeError::new(
                "T-CORE-000003",
                "RemoteRuntimeNotConfigured",
                "Remote runtime not configured",
            )),
            Some(remote_runtime) => {
                let request = self.build_remote_function_request(function, &args);
                match request {
                    Err(err) => Signal::Failure(err),
                    // Handler callbacks are synchronous today. Block only this flow invocation
                    // while the configured remote transport completes its async request.
                    Ok(request) => match block_on(remote_runtime.execute_remote(RemoteExecution {
                        target_service: service.to_string(),
                        request,
                        // Function-thunk settings are always eagerly resolved
                        // to literals (see `build_function_thunk_args`) --
                        // never a `CompiledThunk::Node` reference -- so this
                        // path never mints a sub-flow UUID.
                        sub_flow_activity: None,
                    })) {
                        Ok(result) => remote_result_to_signal(result),
                        Err(err) => Signal::Failure(err),
                    },
                }
            }
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
                NodeResult { signal, frame_id }
            }
            NodeExecutionTarget::Remote { service } => {
                let signal = self
                    .execute_remote_node(node, service, value_store, frame_id)
                    .await;
                NodeResult { signal, frame_id }
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
                NodeResult { signal, frame_id }
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
                NodeResult { signal, frame_id }
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

        if let Err(err) = self.resolve_local_templates(&mut args, value_store, frame_id) {
            return ExecutedNode {
                signal: Signal::Failure(err),
                parameter_results: Vec::new(),
            };
        }

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

        // Shared by every sub-flow UUID minted while resolving this call's
        // parameters (if any): the `sub_flow_execution.*` subscriber bumps
        // it on every lookup+run, so the idle timeout below only fires on
        // genuine inactivity, not because the call is legitimately long-lived.
        let activity = Arc::new(Notify::new());
        let (params, minted_ids) =
            match self.resolve_remote_args(&mut args, value_store, frame_id, &activity) {
                Ok(resolved) => resolved,
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
        let parameter_results = parameter_results_from_remote_params(&params);

        let request = match self.build_remote_request(node, params) {
            Ok(request) => request,
            Err(err) => {
                for id in &minted_ids {
                    self.sub_flow_registry.remove(id);
                }
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

        // Only calls that actually minted a sub-flow reference get the
        // renewable idle timeout; an ordinary call with no sub-flow
        // parameters has no activity to track and keeps today's flat
        // from-the-start deadline (see `NATSRemoteRuntime::execute_remote`).
        let sub_flow_activity = if minted_ids.is_empty() {
            None
        } else {
            Some(Arc::clone(&activity))
        };

        let result = remote_runtime
            .execute_remote(RemoteExecution {
                target_service: service.to_string(),
                request,
                sub_flow_activity,
            })
            .await;

        // The parent call is done (success or failure) -- every sub-flow
        // UUID minted for it is no longer reachable by the action and can
        // be dropped, regardless of how many times (if any) it was actually
        // invoked while the call was outstanding.
        for id in &minted_ids {
            self.sub_flow_registry.remove(id);
        }

        match result {
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
                CompiledArg::Reference(reference) => match value_store.get(reference) {
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
                CompiledArg::Template(template) => {
                    let argument = self.compiled_arg_to_argument(&parameter.arg, value_store)?;
                    self.trace_record_arg(
                        frame_id,
                        ArgTrace {
                            index,
                            kind: ArgKind::Template {
                                references: template.references.len(),
                            },
                            preview: format!("template({} refs)", template.references.len()),
                        },
                    );
                    args.push(argument);
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

    /// Converts one compiled parameter expression into a runtime `Argument`,
    /// recursing into `CompiledTemplate` references. Unlike `build_args`
    /// this performs no tracing -- it's used both for the top-level
    /// `CompiledArg::Template` case and for each nested reference inside it.
    fn compiled_arg_to_argument(
        &self,
        arg: &CompiledArg,
        value_store: &mut ValueStore,
    ) -> Result<Argument, RuntimeError> {
        match arg {
            CompiledArg::Literal(value) => Ok(Argument::Eval(value.clone())),
            CompiledArg::Reference(reference) => match value_store.get(reference) {
                ValueStoreResult::Success(value) => Ok(Argument::Eval(value)),
                ValueStoreResult::Error(err) => Err(err),
                ValueStoreResult::NotFound => Err(RuntimeError::new(
                    "T-CORE-000004",
                    "ReferenceValueNotFound",
                    "Reference not found in execution value store",
                )),
            },
            CompiledArg::Deferred(thunk) => Ok(Argument::Thunk(compiled_thunk_to_argument(thunk))),
            CompiledArg::Template(template) => {
                let mut references = Vec::with_capacity(template.references.len());
                for reference in &template.references {
                    let arg = self.compiled_arg_to_argument(&reference.arg, value_store)?;
                    references.push(TemplateReferenceArgument {
                        signature: reference.signature.clone(),
                        arg: Box::new(arg),
                    });
                }
                Ok(Argument::Template(TemplateArgument {
                    value: template.value.clone(),
                    references,
                }))
            }
        }
    }

    /// Collapses every `Argument::Template` in `args` into `Argument::Eval`
    /// by substituting `${signature}` placeholders with their resolved
    /// values. Local handlers only understand `Eval`/`Thunk`, so this must
    /// run before a local node's handler is invoked -- unlike the remote
    /// path, which forwards the template structure as-is (see
    /// `resolve_remote_args`) so the action can interpolate on its own
    /// schedule and decide when (or whether) to run any nested sub flow.
    fn resolve_local_templates(
        &self,
        args: &mut [Argument],
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<(), RuntimeError> {
        for argument in args.iter_mut() {
            if let Argument::Template(template) = argument {
                let value = self.resolve_template_value(template, value_store, frame_id)?;
                *argument = Argument::Eval(value);
            }
        }
        Ok(())
    }

    fn resolve_template_value(
        &self,
        template: &TemplateArgument,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<Value, RuntimeError> {
        let mut resolved = HashMap::with_capacity(template.references.len());
        for reference in &template.references {
            let value = self.resolve_argument_value(&reference.arg, value_store, frame_id)?;
            resolved.insert(reference.signature.clone(), value);
        }
        Ok(substitute_template(&template.value, &resolved))
    }

    /// Resolves one inline reference to a concrete value, running a deferred
    /// sub-flow thunk synchronously since the interpolated result is needed
    /// immediately.
    fn resolve_argument_value(
        &self,
        argument: &Argument,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
    ) -> Result<Value, RuntimeError> {
        match argument {
            Argument::Eval(value) => Ok(value.clone()),
            Argument::Thunk(thunk) => {
                self.trace_mark_thunk_executed(frame_id, thunk);
                match self.execute_thunk(thunk, value_store).signal {
                    // A reference resolved via `return` inside the sub flow
                    // it points at yields that value the same as `Success`
                    // would -- there's no meaningful difference for the
                    // purpose of filling in one `${signature}` slot.
                    Signal::Success(value) | Signal::Return(value) => Ok(value),
                    Signal::Failure(err) => Err(err),
                    Signal::Stop => Err(RuntimeError::new(
                        "T-CORE-000108",
                        "TemplateReferenceStopped",
                        "Inline reference resolution was stopped before producing a value",
                    )),
                }
            }
            Argument::Template(nested) => {
                self.resolve_template_value(nested, value_store, frame_id)
            }
        }
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

    /// Resolves one remote node's arguments to either a materialized literal
    /// or a minted sub-flow UUID, per positional slot. Returns the resolved
    /// slots alongside every id minted along the way, so the caller can
    /// remove them all once the remote call this request belongs to
    /// resolves (see `execute_remote_node`).
    fn resolve_remote_args(
        &self,
        args: &mut [Argument],
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
        activity: &Arc<Notify>,
    ) -> Result<(Vec<RemoteParam>, Vec<String>), Signal> {
        let mut params = Vec::with_capacity(args.len());
        let mut minted_ids = Vec::new();

        for (index, argument) in args.iter_mut().enumerate() {
            let param = self.resolve_remote_argument(
                argument,
                index,
                value_store,
                frame_id,
                activity,
                &mut minted_ids,
            )?;
            params.push(param);
        }

        Ok((params, minted_ids))
    }

    /// Resolves one remote-call argument (top-level or nested inside a
    /// `Template`'s inline references) into a `RemoteParam`, minting any
    /// sub-flow UUID it needs into `minted_ids`.
    fn resolve_remote_argument(
        &self,
        argument: &mut Argument,
        index: usize,
        value_store: &mut ValueStore,
        frame_id: Option<u64>,
        activity: &Arc<Notify>,
        minted_ids: &mut Vec<String>,
    ) -> Result<RemoteParam, Signal> {
        match argument {
            Argument::Eval(value) => Ok(RemoteParam::Literal(value.clone())),
            // A `CompiledThunk::Node` sub-flow reference destined for a
            // *remote* node's parameter is not resolved here at all --
            // unlike every other thunk in this engine (including the
            // exact same variant reached through the local `build_args`
            // path used by `std::control::if`/`if_else`, which stays
            // eager and synchronous, see `control.rs`), it may need to
            // run zero, one, or many times, driven by the action itself
            // over `ActionSubFlowExecutionRequest` while this call is
            // outstanding. So instead of executing it we mint a UUID
            // and hand the action a `SubFlow` reference it can invoke
            // on its own schedule (see `sub_flow_registry`).
            //
            // `CompiledThunk::Function` (the other `Deferred` variant)
            // is unaffected and keeps executing eagerly below, exactly
            // as before -- only a bare node reference gets this
            // treatment.
            Argument::Thunk(Thunk::Node {
                node_id,
                input_schema,
                output_schema,
            }) => {
                // Not executed, so left exactly as `build_args` already
                // recorded it: `eager: false, executed: false`.
                match self.sub_flow_registry.mint(
                    &self.flow,
                    *node_id,
                    self.execution_id,
                    Arc::clone(activity),
                    value_store.get_current_node_id(),
                    index as i64,
                ) {
                    Some(id) => {
                        minted_ids.push(id.clone());
                        Ok(RemoteParam::SubFlow {
                            execution_identifier: id,
                            input_schema: input_schema.clone(),
                            output_schema: output_schema.clone(),
                        })
                    }
                    None => Err(Signal::Failure(RuntimeError::new(
                        "T-CORE-000001",
                        "NodeNotFound",
                        format!("Node {} not found", node_id),
                    ))),
                }
            }
            Argument::Thunk(thunk @ Thunk::Function(_)) => {
                // Remote execution always receives materialized values for
                // function-thunk args -- this mirrors the pre-existing
                // eager-resolution behavior unchanged.
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
                        Ok(RemoteParam::Literal(value))
                    }
                    // Same unwind rule as local eager params: return exits this call frame only.
                    Signal::Return(value) => Err(Signal::Success(value)),
                    other => Err(other),
                }
            }
            // The template's own text is forwarded to the action as-is
            // (see `ActionLiteralValue`/`ActionInlineReferenceValue`) -- only
            // its references are resolved, recursively, the same way a
            // top-level argument would be. This preserves any nested
            // sub-flow reference as a mintable UUID instead of forcing it to
            // run now, exactly like the top-level `Thunk::Node` case above.
            Argument::Template(template) => {
                let mut references = Vec::with_capacity(template.references.len());
                for reference in &mut template.references {
                    let param = self.resolve_remote_argument(
                        &mut *reference.arg,
                        index,
                        value_store,
                        frame_id,
                        activity,
                        minted_ids,
                    )?;
                    references.push((reference.signature.clone(), param));
                }
                Ok(RemoteParam::Template {
                    value: template.value.clone(),
                    references,
                })
            }
        }
    }

    fn build_remote_request(
        &self,
        node: &CompiledNode,
        params: Vec<RemoteParam>,
    ) -> Result<ActionExecutionRequest, RuntimeError> {
        if node.parameters.len() != params.len() {
            return Err(RuntimeError::new(
                "T-CORE-000005",
                "RemoteParameterMismatch",
                "Remote parameter count mismatch",
            ));
        }

        // Parameters are matched positionally on the receiving end, not by
        // key — `node.parameters` must already be in the function's declared
        // parameter order.
        let parameters = params
            .into_iter()
            .map(|param| ActionNodeValue {
                value: Some(remote_param_to_action_value(param)),
            })
            .collect();

        Ok(ActionExecutionRequest {
            execution_identifier: self.execution_id.to_string(),
            function_identifier: node.handler_id.clone(),
            parameters,
            project_id: self.flow.project_id,
        })
    }

    fn build_remote_function_request(
        &self,
        function: &FunctionThunk,
        args: &[Argument],
    ) -> Result<ActionExecutionRequest, RuntimeError> {
        if function.settings.len() != args.len() {
            return Err(RuntimeError::new(
                "T-CORE-000005",
                "RemoteParameterMismatch",
                "Remote function parameter count mismatch",
            ));
        }

        // Parameters are matched positionally on the receiving end, not by
        // key — `function.settings` must already be in the function's
        // declared parameter order.
        let mut parameters = Vec::with_capacity(args.len());
        for argument in args {
            let Argument::Eval(value) = argument else {
                return Err(RuntimeError::new(
                    "T-CORE-000005",
                    "RemoteParameterMismatch",
                    "Remote function parameters must be evaluated values",
                ));
            };
            parameters.push(ActionNodeValue {
                value: Some(action_node_value::Value::LiteralValue(ActionLiteralValue {
                    value: Some(value.clone()),
                    references: Vec::new(),
                })),
            });
        }

        Ok(ActionExecutionRequest {
            execution_identifier: self.execution_id.to_string(),
            function_identifier: function.identifier.clone(),
            parameters,
            project_id: self.flow.project_id,
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
                // A template not yet collapsed to `Eval` (remote path) has
                // no single materialized value, same as an unresolved thunk.
                Argument::Thunk(_) | Argument::Template(_) => None,
            },
        })
        .collect()
}

/// One resolved remote-call parameter slot: a materialized literal value, a
/// minted sub-flow UUID standing in for a `CompiledThunk::Node` reference the
/// action may invoke later, or a literal template forwarded to the action
/// with its own references resolved the same way (see `resolve_remote_args`).
enum RemoteParam {
    Literal(Value),
    SubFlow {
        execution_identifier: String,
        input_schema: Option<Struct>,
        output_schema: Option<Struct>,
    },
    Template {
        value: Value,
        references: Vec<(String, RemoteParam)>,
    },
}

fn remote_param_to_action_value(param: RemoteParam) -> action_node_value::Value {
    match param {
        RemoteParam::Literal(value) => action_node_value::Value::LiteralValue(ActionLiteralValue {
            value: Some(value),
            references: Vec::new(),
        }),
        RemoteParam::SubFlow {
            execution_identifier,
            input_schema,
            output_schema,
        } => action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
            execution_identifier,
            input_schema,
            output_schema,
        }),
        RemoteParam::Template { value, references } => {
            let references = references
                .into_iter()
                .map(|(signature, param)| ActionInlineReferenceValue {
                    signature,
                    value: Some(ActionNodeValue {
                        value: Some(remote_param_to_action_value(param)),
                    }),
                })
                .collect();
            action_node_value::Value::LiteralValue(ActionLiteralValue {
                value: Some(value),
                references,
            })
        }
    }
}

fn parameter_results_from_remote_params(
    params: &[RemoteParam],
) -> Vec<NodeParameterNodeExecutionResult> {
    params
        .iter()
        .map(|param| NodeParameterNodeExecutionResult {
            value: match param {
                RemoteParam::Literal(value) => Some(value.clone()),
                // No literal value was materialized for a minted sub-flow
                // reference or an unresolved template -- same convention as
                // an unresolved `Argument::Thunk` in `parameter_results_from_args`.
                RemoteParam::SubFlow { .. } | RemoteParam::Template { .. } => None,
            },
        })
        .collect()
}

fn compiled_thunk_to_argument(thunk: &CompiledThunk) -> Thunk {
    match thunk {
        CompiledThunk::Node {
            node_id,
            input_schema,
            output_schema,
        } => Thunk::Node {
            node_id: *node_id,
            input_schema: input_schema.clone(),
            output_schema: output_schema.clone(),
        },
        CompiledThunk::Function {
            identifier,
            execution_target,
            parameter_index,
            settings,
        } => Thunk::Function(FunctionThunk {
            identifier: identifier.clone(),
            execution_target: execution_target.clone(),
            parameter_index: *parameter_index,
            settings: settings.clone(),
        }),
    }
}

fn remote_result_to_signal(result: TucanaNodeExecutionResult) -> Signal {
    match result.result {
        Some(TucanaNodeResult::Success(value)) => Signal::Success(value),
        Some(TucanaNodeResult::Error(error)) => {
            Signal::Failure(RuntimeError::from_tucana_error(&error))
        }
        None => Signal::Failure(RuntimeError::new(
            "T-CORE-000006",
            "NodeExecutionResultMissingOutcome",
            "Remote function execution result is missing success/error outcome",
        )),
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

    match value_store.get(&reference) {
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

/// Substitutes every `${signature}` placeholder found in a (possibly nested)
/// string inside `value` with its resolved value from `resolved`, per the
/// `LiteralValue`/`ActionLiteralValue` doc comments in tucana. A string that
/// is *exactly* one placeholder (nothing else around it) is replaced with
/// the resolved value verbatim, preserving its type (e.g. a number reference
/// used as a whole parameter value stays a number); a placeholder embedded
/// in a larger string is spliced in as text (see `stringify_for_template`).
fn substitute_template(value: &Value, resolved: &HashMap<String, Value>) -> Value {
    match value.kind.as_ref() {
        Some(Kind::StringValue(s)) => substitute_string_template(s, resolved),
        Some(Kind::StructValue(struct_value)) => Value {
            kind: Some(Kind::StructValue(Struct {
                fields: struct_value
                    .fields
                    .iter()
                    .map(|(key, value)| (key.clone(), substitute_template(value, resolved)))
                    .collect(),
            })),
        },
        Some(Kind::ListValue(list)) => Value {
            kind: Some(Kind::ListValue(ListValue {
                values: list
                    .values
                    .iter()
                    .map(|value| substitute_template(value, resolved))
                    .collect(),
            })),
        },
        _ => value.clone(),
    }
}

fn substitute_string_template(raw: &str, resolved: &HashMap<String, Value>) -> Value {
    if let Some(signature) = sole_placeholder(raw) {
        return resolved.get(signature).cloned().unwrap_or_else(null_value);
    }

    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let Some(end_rel) = rest[start + 2..].find('}') else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let end = start + 2 + end_rel;
        let signature = &rest[start + 2..end];
        match resolved.get(signature) {
            Some(value) => out.push_str(&stringify_for_template(value)),
            // Unmatched placeholder -- left verbatim rather than silently dropped.
            None => out.push_str(&rest[start..=end]),
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);

    Value {
        kind: Some(Kind::StringValue(out)),
    }
}

/// A string consisting of exactly one `${signature}` placeholder and
/// nothing else -- lets a whole-value reference preserve its original type
/// instead of being stringified (see `substitute_template`).
fn sole_placeholder(raw: &str) -> Option<&str> {
    let inner = raw.strip_prefix("${")?.strip_suffix('}')?;
    if inner.contains("${") || inner.contains('}') {
        None
    } else {
        Some(inner)
    }
}

fn stringify_for_template(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::StringValue(s)) => s.clone(),
        Some(Kind::NumberValue(v)) => crate::value::number_to_string(v),
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::NullValue(_)) | None => String::new(),
        Some(Kind::StructValue(_)) | Some(Kind::ListValue(_)) => format_value_json(value),
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
