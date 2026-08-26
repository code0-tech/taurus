//! Public runtime execution API.
//!
//! This module is the new entrypoint for flow execution from external crates.
//! It executes compiled flow plans via the runtime engine executor loop.

mod compiler;
mod executor;
pub(crate) mod model;
mod sub_flow_registry;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use futures_lite::future::block_on;
use lru::LruCache;
use prost::Message as _;
use tucana::shared::value::Kind;
use tucana::shared::{ExecutionFlow, NodeExecutionResult, NodeFunction, Value};

use crate::handler::registry::FunctionStore;
use crate::runtime::execution::trace::TraceRun;
use crate::runtime::execution::value_store::ValueStore;
use crate::runtime::remote::RemoteRuntime;
use crate::types::exit_reason::ExitReason;
use crate::types::signal::Signal;
use compiler::compile_flow;
use model::CompiledFlow;
use sub_flow_registry::SubFlowRegistry;

/// Unique identifier for one top-level flow execution.
pub type ExecutionId = uuid::Uuid;

/// Number of distinct compiled flows kept warm by default; see
/// [`ExecutionEngine::with_compiled_flow_cache_limits`].
pub const DEFAULT_COMPILED_FLOW_CACHE_CAPACITY: usize = 512;

/// Default total memory budget for the compiled-flow cache. A flow's raw
/// protobuf size is a lower bound on its compiled footprint, not an
/// estimate of it -- see `COMPILED_SIZE_WEIGHT_MULTIPLIER` -- so this caps
/// *estimated* compiled bytes, not wire bytes.
pub const DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES: usize = 256 * 1024 * 1024;

/// `CompiledFlow` re-derives a graph of owned `String`s (handler ids,
/// parameter ids, template signatures, remote service names), a `Vec` per
/// node, a secondary `HashMap<i64, usize>` index alongside the node list,
/// and boxed nested args for templates -- all heap allocations the raw
/// encoded protobuf bytes don't pay for. There's no exact measurement of
/// this in the codebase; 3x is a middle-of-the-road estimate for "many
/// small strings and nested collections" workloads (plausible range 2-5x)
/// used only to size the cache's byte budget conservatively.
const COMPILED_SIZE_WEIGHT_MULTIPLIER: usize = 3;

/// `(project_id, start_node_id, content_hash_of_node_functions)`.
///
/// `compile_flow`'s output depends only on these three inputs, so hashing
/// the encoded `NodeFunction` bytes (rather than requiring `Hash`/`Eq` on
/// the protobuf types, which they don't derive) gives a correctness-safe
/// key: any edit to a node, its parameters, or the graph shape changes the
/// encoded bytes and therefore the key, forcing a recompile.
type CompiledFlowCacheKey = (i64, i64, u64);

/// Cache key plus an estimated in-memory weight (see
/// `COMPILED_SIZE_WEIGHT_MULTIPLIER`), both derived from a single pass over
/// the encoded node bytes.
fn compiled_flow_cache_key_and_weight(
    project_id: i64,
    start_node_id: i64,
    nodes: &[NodeFunction],
) -> (CompiledFlowCacheKey, usize) {
    let mut buf = Vec::new();
    for node in nodes {
        node.encode(&mut buf)
            .expect("Vec<u8> buffer writes are infallible");
    }
    let weight_bytes = buf.len().saturating_mul(COMPILED_SIZE_WEIGHT_MULTIPLIER);
    let mut hasher = DefaultHasher::new();
    buf.hash(&mut hasher);
    ((project_id, start_node_id, hasher.finish()), weight_bytes)
}

#[cfg(test)]
fn compiled_flow_cache_key(
    project_id: i64,
    start_node_id: i64,
    nodes: &[NodeFunction],
) -> CompiledFlowCacheKey {
    compiled_flow_cache_key_and_weight(project_id, start_node_id, nodes).0
}

struct CachedCompiledFlow {
    plan: Arc<CompiledFlow>,
    weight_bytes: usize,
}

/// LRU cache of compiled flows, bounded by both entry count and an
/// estimated total byte weight -- whichever limit is hit first evicts the
/// least-recently-used entry. In-process only, not shared across replicas
/// or persisted across restarts: a network hop would cost more than the
/// compile it's avoiding (see `ExecutionEngine::with_compiled_flow_cache_limits`).
struct CompiledFlowCache {
    entries: LruCache<CompiledFlowCacheKey, CachedCompiledFlow>,
    total_bytes: usize,
    max_bytes: usize,
}

impl CompiledFlowCache {
    fn new(capacity: NonZeroUsize, max_bytes: usize) -> Self {
        Self {
            entries: LruCache::new(capacity),
            total_bytes: 0,
            max_bytes,
        }
    }

    fn get(&mut self, key: &CompiledFlowCacheKey) -> Option<Arc<CompiledFlow>> {
        self.entries.get(key).map(|entry| Arc::clone(&entry.plan))
    }

    /// No-op if `weight_bytes` alone exceeds the whole budget -- caching a
    /// single flow that big would just immediately evict everything else
    /// (including itself, next insert), so it's simplest to skip caching it
    /// and let it recompile every time instead.
    fn insert(&mut self, key: CompiledFlowCacheKey, plan: Arc<CompiledFlow>, weight_bytes: usize) {
        if weight_bytes > self.max_bytes {
            return;
        }
        if let Some((_, evicted)) = self.entries.push(key, CachedCompiledFlow { plan, weight_bytes }) {
            self.total_bytes -= evicted.weight_bytes;
        }
        self.total_bytes += weight_bytes;
        while self.total_bytes > self.max_bytes {
            match self.entries.pop_lru() {
                Some((_, evicted)) => self.total_bytes -= evicted.weight_bytes,
                None => break,
            }
        }
    }

    #[cfg(test)]
    fn peek(&self, key: &CompiledFlowCacheKey) -> Option<Arc<CompiledFlow>> {
        self.entries.peek(key).map(|entry| Arc::clone(&entry.plan))
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Runtime engine entrypoint used by runtime binaries and CLI tools.
pub struct ExecutionEngine {
    handlers: FunctionStore,
    /// Registry of sub-flow node ranges minted while a remote node call is
    /// outstanding -- shared with every `EngineExecutor` (to mint) and with
    /// the `sub_flow_execution.*` NATS subscriber (via `execute_sub_flow`,
    /// to look up and run). See `sub_flow_registry` for the full rationale.
    sub_flow_registry: SubFlowRegistry,
    /// `None` means the cache is disabled (zero entry capacity or zero byte
    /// budget) -- every execution recompiles, matching pre-cache behavior.
    compiled_flow_cache: Mutex<Option<CompiledFlowCache>>,
}

/// Full result of one engine execution, including per-node results for reporting.
#[derive(Debug, Clone)]
pub struct EngineExecutionReport {
    pub signal: Signal,
    pub exit_reason: ExitReason,
    pub node_execution_results: Vec<NodeExecutionResult>,
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine {
    /// Build a new execution engine with default handler registry and a
    /// compiled-flow cache bounded by both
    /// [`DEFAULT_COMPILED_FLOW_CACHE_CAPACITY`] entries and
    /// [`DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES`] estimated bytes.
    pub fn new() -> Self {
        Self::with_compiled_flow_cache_limits(
            DEFAULT_COMPILED_FLOW_CACHE_CAPACITY,
            DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES,
        )
    }

    /// Build a new execution engine with a compiled-flow cache bounded to
    /// `cache_capacity` entries, using the default byte budget
    /// ([`DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES`]). Pass `0` to disable the
    /// cache and recompile every execution, as before.
    pub fn with_compiled_flow_cache_capacity(cache_capacity: usize) -> Self {
        Self::with_compiled_flow_cache_limits(cache_capacity, DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES)
    }

    /// Build a new execution engine with a compiled-flow cache bounded by
    /// *both* `cache_capacity` entries and `cache_max_bytes` estimated
    /// total bytes (see `COMPILED_SIZE_WEIGHT_MULTIPLIER`) -- whichever
    /// limit is hit first evicts the least-recently-used entry. Pass `0`
    /// for either to disable the cache and recompile every execution.
    pub fn with_compiled_flow_cache_limits(cache_capacity: usize, cache_max_bytes: usize) -> Self {
        let cache = if cache_max_bytes == 0 {
            None
        } else {
            NonZeroUsize::new(cache_capacity).map(|cap| CompiledFlowCache::new(cap, cache_max_bytes))
        };
        Self {
            handlers: FunctionStore::default(),
            sub_flow_registry: SubFlowRegistry::new(),
            compiled_flow_cache: Mutex::new(cache),
        }
    }

    /// Execute an `ExecutionFlow` and return the final signal plus per-node execution results.
    pub fn execute_flow_report(
        &self,
        execution_id: &str,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_flow_report_async(execution_id, flow, remote, with_trace))
    }

    /// Execute an `ExecutionFlow` asynchronously and return per-node results.
    ///
    /// `execution_id` is reused as the `execution_identifier` on any remote
    /// call this flow makes back into an action (e.g. a `respond`-style
    /// callback) — the action correlates that id against the one it used to
    /// originally trigger this flow, so it must match exactly, not be a
    /// freshly generated id per remote call.
    pub async fn execute_flow_report_async(
        &self,
        execution_id: &str,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        self.execute_graph_with_project_id_report_async(
            execution_id,
            flow.project_id,
            flow.starting_node_id,
            flow.node_functions,
            flow.input_value,
            remote,
            with_trace,
        )
        .await
    }

    /// Execute a graph described by node list and start node.
    pub fn execute_graph(
        &self,
        execution_id: &str,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = block_on(self.execute_graph_with_project_id_report_async(
            execution_id,
            0,
            start_node_id,
            node_functions,
            flow_input,
            remote,
            with_trace,
        ));
        (report.signal, report.exit_reason)
    }

    /// Execute a graph and return the final signal plus per-node execution results.
    pub fn execute_graph_report(
        &self,
        execution_id: &str,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_graph_with_project_id_report_async(
            execution_id,
            0,
            start_node_id,
            node_functions,
            flow_input,
            remote,
            with_trace,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_graph_with_project_id_report_async(
        &self,
        execution_id: &str,
        project_id: i64,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        let mut value_store = ValueStore::new(flow_input.unwrap_or_default(), with_trace);

        let (cache_key, cache_weight) =
            compiled_flow_cache_key_and_weight(project_id, start_node_id, &node_functions);
        let cached = self
            .compiled_flow_cache
            .lock()
            .unwrap()
            .as_mut()
            .and_then(|cache| cache.get(&cache_key));

        // Wrapped in `Arc` here (whether freshly compiled or cloned from
        // cache), so that minting a sub-flow registry entry is a cheap
        // refcount bump instead of a deep clone of the node graph (see
        // `sub_flow_registry`).
        let compiled = match cached {
            Some(plan) => plan,
            None => {
                let plan = match compile_flow(project_id, start_node_id, node_functions) {
                    Ok(plan) => Arc::new(plan),
                    Err(err) => {
                        let runtime_error = err.as_runtime_error();
                        let signal = Signal::Failure(runtime_error);
                        return EngineExecutionReport {
                            signal,
                            exit_reason: ExitReason::Failure,
                            node_execution_results: Vec::new(),
                        };
                    }
                };
                if let Some(cache) = self.compiled_flow_cache.lock().unwrap().as_mut() {
                    cache.insert(cache_key, Arc::clone(&plan), cache_weight);
                }
                plan
            }
        };
        let start_idx = compiled.start_idx;

        let (signal, trace_run) = executor::execute_compiled_from(
            execution_id,
            &compiled,
            start_idx,
            &self.handlers,
            &mut value_store,
            remote,
            with_trace,
            self.sub_flow_registry.clone(),
        )
        .await;
        Self::finish_report(signal, trace_run, &mut value_store, with_trace)
    }

    /// Run a previously minted sub-flow node range (see `SubFlowRegistry`).
    ///
    /// `parameters` are the action-supplied positional values from
    /// `ActionSubFlowExecutionRequest.parameters` -- bound the same way a
    /// local consumer callback binds them for a native `for_each`/`map`/etc.
    /// (see `functions/array.rs::run_with_unary_input`): each positional
    /// value is seeded as `InputType{node_id: caller_node_id, parameter_index:
    /// caller_parameter_index, input_index}`, so `Target::InputType`
    /// references inside the sub-flow's node range -- which is exactly what
    /// the compiler emits for a value the sub-flow was invoked with -- find
    /// them keyed the same way regardless of whether the callback ran
    /// in-process or, as here, standalone in response to an
    /// `ActionSubFlowExecutionRequest`.
    ///
    /// Returns `None` if `execution_identifier` doesn't match any pending
    /// sub-flow -- already completed (parent call resolved and the entry
    /// was removed), never minted, or minted by a process instance that has
    /// since restarted (the registry is in-memory only).
    pub async fn execute_sub_flow(
        &self,
        execution_identifier: &str,
        parameters: Vec<Value>,
        remote: Option<&dyn RemoteRuntime>,
        with_trace: bool,
    ) -> Option<EngineExecutionReport> {
        let pending = self.sub_flow_registry.get(execution_identifier)?;
        // Bump the parent call's idle-timeout activity marker: this lookup
        // is itself proof the parent call is still being actively driven.
        pending.activity.notify_one();

        let mut value_store = ValueStore::new(
            Value {
                kind: Some(Kind::NullValue(0)),
            },
            with_trace,
        );
        for (input_index, value) in parameters.into_iter().enumerate() {
            value_store.insert_input_type(
                tucana::shared::InputType {
                    node_id: pending.caller_node_id,
                    parameter_index: pending.caller_parameter_index,
                    input_index: input_index as i64,
                },
                value,
            );
        }

        // Deliberately *not* `pending.parent_execution_id`: if a node inside
        // this sub-flow's own node range is itself dispatched remotely, it
        // needs a fresh, unique `execution_identifier` for its own
        // `ActionExecutionRequest`. Aquila's `PendingReplyStore`
        // (`nats_bridge.rs`) is a flat `HashMap<execution_identifier,
        // reply_subject>` with last-write-wins semantics on collision — and
        // the parent's own remote call is *guaranteed* to still be
        // outstanding under `parent_execution_id` for as long as this
        // sub-flow run can happen at all (that's the entire premise of
        // sub-flow execution). Reusing it here would silently clobber the
        // parent's pending-reply entry the moment this run makes its own
        // remote call, cross-wiring both calls' eventual replies. The
        // action doesn't need this id to equal the parent's to correlate
        // the sub-flow run back to its session -- it already has that via
        // the sub-flow's own minted id (`execution_identifier` above),
        // which travelled to it in `ActionSubFlowExecutionRequest`.
        let run_execution_id = uuid::Uuid::new_v4().to_string();
        log::debug!(
            "Running sub flow execution_identifier={} for parent_execution_id={} as run_execution_id={}",
            execution_identifier,
            pending.parent_execution_id,
            run_execution_id
        );

        let (signal, trace_run) = executor::execute_compiled_from(
            &run_execution_id,
            &pending.flow,
            pending.start_idx,
            &self.handlers,
            &mut value_store,
            remote,
            with_trace,
            self.sub_flow_registry.clone(),
        )
        .await;
        Some(Self::finish_report(
            signal,
            trace_run,
            &mut value_store,
            with_trace,
        ))
    }

    fn finish_report(
        signal: Signal,
        trace_run: Option<TraceRun>,
        value_store: &mut ValueStore,
        with_trace: bool,
    ) -> EngineExecutionReport {
        if with_trace && let Some(trace_run) = trace_run {
            println!(
                "{}",
                crate::runtime::execution::render::render_trace(&trace_run)
            );
        }
        let exit_reason = signal.exit_reason();
        EngineExecutionReport {
            signal,
            exit_reason,
            node_execution_results: value_store.node_execution_results(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::argument::Argument;
    use crate::handler::registry::{FunctionRegistration, FunctionStore, ThunkRunner};
    use crate::runtime::execution::value_store::ValueStore;
    use crate::runtime::remote::{RemoteExecution, RemoteRuntime};
    use crate::types::exit_reason::ExitReason;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tucana::aquila::{
        ActionExecutionRequest, ActionLiteralValue, ActionNodeSubFlowValue, action_node_value,
    };
    use tucana::shared::{
        InputType, ListValue, LiteralValue, NodeExecutionResult, NodeParameter, NodeValue,
        ReferenceValue, Struct, SubFlow, SubFlowFunction, SubFlowSetting, Value,
        node_execution_result, node_value, reference_value, sub_flow::ExecutionReference,
        value::Kind,
    };

    fn literal_param(database_id: i64, runtime_parameter_id: &str, value: Value) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::LiteralValue(LiteralValue {
                    value: Some(value),
                    references: Vec::new(),
                })),
            }),
            cast: None,
        }
    }

    fn thunk_param(database_id: i64, runtime_parameter_id: &str, node_id: i64) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::SubFlow(SubFlow {
                    input_schema: None,
                    output_schema: None,
                    signature: String::new(),
                    settings: Vec::new(),
                    execution_reference: Some(ExecutionReference::StartingNodeId(node_id)),
                })),
            }),
            cast: None,
        }
    }

    fn function_thunk_param(
        database_id: i64,
        runtime_parameter_id: &str,
        function_identifier: &str,
        settings: Vec<SubFlowSetting>,
    ) -> NodeParameter {
        function_thunk_param_with_source(
            database_id,
            runtime_parameter_id,
            function_identifier,
            None,
            settings,
        )
    }

    fn function_thunk_param_with_source(
        database_id: i64,
        runtime_parameter_id: &str,
        function_identifier: &str,
        definition_source: Option<&str>,
        settings: Vec<SubFlowSetting>,
    ) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::SubFlow(SubFlow {
                    input_schema: None,
                    output_schema: None,
                    signature: String::new(),
                    settings,
                    execution_reference: Some(ExecutionReference::Function(SubFlowFunction {
                        function_identifier: function_identifier.to_string(),
                        definition_source: definition_source.map(str::to_string),
                    })),
                })),
            }),
            cast: None,
        }
    }

    fn subflow_setting(
        identifier: &str,
        default_value: Option<Value>,
        optional: bool,
        hidden: bool,
    ) -> SubFlowSetting {
        SubFlowSetting {
            identifier: identifier.to_string(),
            default_value,
            optional: Some(optional),
            hidden: Some(hidden),
        }
    }

    fn node_result_ref_param(
        database_id: i64,
        runtime_parameter_id: &str,
        node_id: i64,
    ) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                    target: Some(reference_value::Target::NodeId(node_id)),
                    paths: Vec::new(),
                })),
            }),
            cast: None,
        }
    }

    fn node(
        database_id: i64,
        runtime_function_id: &str,
        parameters: Vec<NodeParameter>,
        next_node_id: Option<i64>,
    ) -> NodeFunction {
        NodeFunction {
            database_id: Some(database_id),
            runtime_function_id: runtime_function_id.to_string(),
            parameters,
            next_node_id,
            definition_source: Some("taurus".to_string()),
        }
    }

    fn int_value(value: i64) -> Value {
        crate::value::value_from_i64(value)
    }

    fn string_value(value: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(value.to_string())),
        }
    }

    fn null_value() -> Value {
        Value {
            kind: Some(Kind::NullValue(0)),
        }
    }

    fn empty_struct_value() -> Value {
        Value {
            kind: Some(Kind::StructValue(Struct {
                fields: std::collections::HashMap::new(),
            })),
        }
    }

    fn list_value(values: Vec<Value>) -> Value {
        Value {
            kind: Some(Kind::ListValue(ListValue { values })),
        }
    }

    fn expect_success(signal: Signal) -> Value {
        match signal {
            Signal::Success(value) => value,
            other => panic!("expected success, got {:?}", other),
        }
    }

    fn assert_node_result_id(result: &NodeExecutionResult, expected_id: i64) {
        assert_eq!(
            result.id,
            Some(node_execution_result::Id::NodeId(expected_id))
        );
    }

    fn assert_function_result_id(result: &NodeExecutionResult, expected_id: &str) {
        assert_eq!(
            result.id,
            Some(node_execution_result::Id::FunctionIdentifier(
                expected_id.to_string()
            ))
        );
    }

    fn sleep_handler(
        _args: &[Argument],
        _ctx: &mut ValueStore,
        _run: &mut ThunkRunner<'_>,
    ) -> Signal {
        std::thread::sleep(Duration::from_micros(2_000));
        Signal::Success(null_value())
    }

    fn echo_first_arg_handler(
        args: &[Argument],
        _ctx: &mut ValueStore,
        _run: &mut ThunkRunner<'_>,
    ) -> Signal {
        match args.first() {
            Some(Argument::Eval(value)) => Signal::Success(value.clone()),
            _ => Signal::Failure(crate::types::errors::runtime_error::RuntimeError::new(
                "T-TEST-000001",
                "MissingEchoArgument",
                "expected first eager argument",
            )),
        }
    }

    #[derive(Clone)]
    struct StubRemoteRuntime {
        result: NodeExecutionResult,
        target_services: Option<Arc<Mutex<Vec<String>>>>,
        project_ids: Option<Arc<Mutex<Vec<i64>>>>,
        requests: Option<Arc<Mutex<Vec<ActionExecutionRequest>>>>,
    }

    #[async_trait]
    impl RemoteRuntime for StubRemoteRuntime {
        async fn execute_remote(
            &self,
            execution: RemoteExecution,
        ) -> Result<NodeExecutionResult, crate::types::errors::runtime_error::RuntimeError>
        {
            if let Some(target_services) = &self.target_services {
                target_services
                    .lock()
                    .expect("target service recorder should not be poisoned")
                    .push(execution.target_service);
            }
            if let Some(project_ids) = &self.project_ids {
                project_ids
                    .lock()
                    .expect("project id recorder should not be poisoned")
                    .push(execution.request.project_id);
            }
            if let Some(requests) = &self.requests {
                requests
                    .lock()
                    .expect("request recorder should not be poisoned")
                    .push(execution.request.clone());
            }

            Ok(self.result.clone())
        }
    }

    fn input_type_ref_param(
        database_id: i64,
        runtime_parameter_id: &str,
        node_id: i64,
        parameter_index: i64,
        input_index: i64,
    ) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::ReferenceValue(ReferenceValue {
                    target: Some(reference_value::Target::InputType(InputType {
                        node_id,
                        parameter_index,
                        input_index,
                    })),
                    paths: Vec::new(),
                })),
            }),
            cast: None,
        }
    }

    #[test]
    fn eager_thunk_return_unwinds_one_level_and_continues_with_parent_next() {
        let engine = ExecutionEngine::new();

        // Node 10 is used as eager parameter thunk by node 2.
        // It returns 42 and must not continue to its own next node.
        let return_node = node(
            10,
            "std::control::return",
            vec![literal_param(100, "value", int_value(42))],
            Some(12),
        );

        // If this node ever executes, the test expectation below will fail.
        let unreachable_after_return = node(12, "std::number::add", vec![], None);

        // Parent node A (id=2): eager arg is node 10.
        let parent = node(
            2,
            "std::number::add",
            vec![
                thunk_param(200, "lhs", 10),
                literal_param(201, "rhs", int_value(1)),
            ],
            Some(3),
        );

        // Next node B (id=3): depends on A result and adds 1.
        let next = node(
            3,
            "std::number::add",
            vec![
                node_result_ref_param(300, "lhs", 2),
                literal_param(301, "rhs", int_value(1)),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph(
            "test",
            2,
            vec![parent, next, return_node, unreachable_after_return],
            None,
            None,
            false,
        );

        assert_eq!(reason, ExitReason::Success);
        match signal {
            Signal::Success(Value {
                kind: Some(Kind::NumberValue(number)),
            }) => match number.number {
                Some(tucana::shared::number_value::Number::Integer(v)) => assert_eq!(v, 43),
                other => panic!("expected integer result 43, got {:?}", other),
            },
            other => panic!("expected success with value 43, got {:?}", other),
        }
    }

    #[test]
    fn return_inside_map_callback_returns_callback_value_only() {
        let engine = ExecutionEngine::new();

        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(
                    100,
                    "list",
                    list_value(vec![
                        string_value("age"),
                        string_value("email"),
                        string_value("username"),
                    ]),
                ),
                thunk_param(101, "transform", 2),
            ],
            None,
        );

        let is_equal_node = node(
            2,
            "std::text::is_equal",
            vec![
                input_type_ref_param(200, "first", 1, 1, 0),
                literal_param(201, "second", string_value("username")),
            ],
            Some(3),
        );

        let if_node = node(
            3,
            "std::control::if",
            vec![
                node_result_ref_param(300, "condition", 2),
                thunk_param(301, "runnable", 4),
            ],
            Some(5),
        );

        let return_item_node = node(
            4,
            "std::control::return",
            vec![input_type_ref_param(400, "value", 1, 1, 0)],
            None,
        );

        let return_null_node = node(
            5,
            "std::control::return",
            vec![literal_param(500, "value", null_value())],
            None,
        );

        let (signal, reason) = engine.execute_graph(
            "test",
            1,
            vec![
                map_node,
                is_equal_node,
                if_node,
                return_item_node,
                return_null_node,
            ],
            None,
            None,
            false,
        );

        assert_eq!(reason, ExitReason::Success);
        match signal {
            Signal::Success(Value {
                kind: Some(Kind::ListValue(ListValue { values })),
            }) => {
                assert_eq!(
                    values,
                    vec![null_value(), null_value(), string_value("username")]
                );
            }
            other => panic!(
                "expected Success([null, null, \"username\"]), got {:?}",
                other
            ),
        }
    }

    #[test]
    fn function_subflow_map_executes_function_identifier_with_iteration_input() {
        let engine = ExecutionEngine::new();

        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![int_value(1), int_value(2)])),
                function_thunk_param(
                    101,
                    "transform",
                    "std::number::add",
                    vec![
                        subflow_setting("lhs", None, false, false),
                        subflow_setting("rhs", Some(int_value(2)), false, true),
                    ],
                ),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![map_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            expect_success(signal),
            list_value(vec![int_value(3), int_value(4)])
        );
    }

    #[test]
    fn function_subflow_map_routes_non_local_function_to_remote_runtime() {
        let engine = ExecutionEngine::new();
        let target_services = Arc::new(Mutex::new(Vec::new()));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::FunctionIdentifier(
                    "remote::add".to_string(),
                )),
                result: Some(node_execution_result::Result::Success(int_value(99))),
            },
            target_services: Some(Arc::clone(&target_services)),
            project_ids: None,
            requests: Some(Arc::clone(&requests)),
        };
        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![int_value(1), int_value(2)])),
                function_thunk_param_with_source(
                    101,
                    "transform",
                    "remote::add",
                    Some("action.example"),
                    vec![
                        subflow_setting("lhs", None, false, false),
                        subflow_setting("rhs", Some(int_value(2)), false, true),
                    ],
                ),
            ],
            None,
        );
        let flow = ExecutionFlow {
            flow_id: 10,
            project_id: 42,
            starting_node_id: 1,
            node_functions: vec![map_node],
            input_value: None,
        };

        let report = engine.execute_flow_report("test", flow, Some(&remote), false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(
            expect_success(report.signal),
            list_value(vec![int_value(99), int_value(99)])
        );
        assert_eq!(
            *target_services
                .lock()
                .expect("target service recorder should not be poisoned"),
            vec!["example".to_string(), "example".to_string()]
        );

        let requests = requests
            .lock()
            .expect("request recorder should not be poisoned");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].function_identifier, "remote::add");
        assert_eq!(requests[0].project_id, 42);
        let first_parameters = &requests[0].parameters;
        assert_eq!(first_parameters.len(), 2);
        assert_eq!(
            first_parameters[0].value,
            Some(action_node_value::Value::LiteralValue(ActionLiteralValue {
                value: Some(int_value(1)),
                references: Vec::new(),
            }))
        );
        assert_eq!(
            first_parameters[1].value,
            Some(action_node_value::Value::LiteralValue(ActionLiteralValue {
                value: Some(int_value(2)),
                references: Vec::new(),
            }))
        );

        let function_results: Vec<_> = report
            .node_execution_results
            .iter()
            .filter(|result| {
                matches!(
                    result.id,
                    Some(node_execution_result::Id::FunctionIdentifier(_))
                )
            })
            .collect();
        assert_eq!(function_results.len(), 2);
        for result in function_results {
            assert_function_result_id(result, "remote::add");
        }
    }

    #[test]
    fn remote_function_subflow_fails_without_remote_runtime() {
        let engine = ExecutionEngine::new();
        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![int_value(1)])),
                function_thunk_param_with_source(
                    101,
                    "transform",
                    "remote::identity",
                    Some("remote-service"),
                    vec![subflow_setting("value", None, false, false)],
                ),
            ],
            None,
        );

        let report = engine.execute_graph_report("test", 1, vec![map_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Failure);
        match report.signal {
            Signal::Failure(err) => assert_eq!(err.code, "T-CORE-000003"),
            other => panic!("expected missing remote runtime failure, got {:?}", other),
        }
    }

    #[test]
    fn remote_function_subflow_rejects_empty_action_service() {
        let engine = ExecutionEngine::new();
        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![int_value(1)])),
                function_thunk_param_with_source(
                    101,
                    "transform",
                    "remote::identity",
                    Some("action."),
                    vec![subflow_setting("value", None, false, false)],
                ),
            ],
            None,
        );

        let report = engine.execute_graph_report("test", 1, vec![map_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Failure);
        assert!(report.node_execution_results.is_empty());
        match report.signal {
            Signal::Failure(err) => assert_eq!(err.code, "T-CORE-000106"),
            other => panic!(
                "expected invalid definition source failure, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn function_subflow_filter_executes_predicate_identifier() {
        let engine = ExecutionEngine::new();

        let filter_node = node(
            1,
            "std::list::filter",
            vec![
                literal_param(
                    100,
                    "list",
                    list_value(vec![int_value(1), int_value(4), int_value(7)]),
                ),
                function_thunk_param(
                    101,
                    "predicate",
                    "std::number::is_greater",
                    vec![
                        subflow_setting("lhs", None, false, false),
                        subflow_setting("rhs", Some(int_value(3)), false, true),
                    ],
                ),
            ],
            None,
        );

        let (signal, reason) =
            engine.execute_graph("test", 1, vec![filter_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            expect_success(signal),
            list_value(vec![int_value(4), int_value(7)])
        );
    }

    #[test]
    fn function_subflow_default_replaces_null_callback_input() {
        let engine = ExecutionEngine::new();

        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![null_value(), int_value(5)])),
                function_thunk_param(
                    101,
                    "transform",
                    "std::control::value",
                    vec![subflow_setting("value", Some(int_value(9)), false, false)],
                ),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![map_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            expect_success(signal),
            list_value(vec![int_value(9), int_value(5)])
        );
    }

    #[test]
    fn function_subflow_hidden_setting_always_uses_default() {
        let engine = ExecutionEngine::new();

        let map_node = node(
            1,
            "std::list::map",
            vec![
                literal_param(100, "list", list_value(vec![int_value(1), int_value(2)])),
                function_thunk_param(
                    101,
                    "transform",
                    "std::control::value",
                    vec![subflow_setting("value", Some(int_value(9)), false, true)],
                ),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![map_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            expect_success(signal),
            list_value(vec![int_value(9), int_value(9)])
        );
    }

    #[test]
    fn function_subflow_optional_missing_setting_uses_null() {
        let engine = ExecutionEngine::new();

        let if_node = node(
            1,
            "std::control::if",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                ),
                function_thunk_param(
                    101,
                    "runnable",
                    "std::control::value",
                    vec![subflow_setting("value", None, true, false)],
                ),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![if_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), null_value());
    }

    #[test]
    fn function_subflow_required_missing_setting_fails() {
        let engine = ExecutionEngine::new();

        let if_node = node(
            1,
            "std::control::if",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                ),
                function_thunk_param(
                    101,
                    "runnable",
                    "std::control::value",
                    vec![subflow_setting("value", None, false, false)],
                ),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![if_node], None, None, false);

        assert_eq!(reason, ExitReason::Failure);
        match signal {
            Signal::Failure(err) => assert_eq!(err.code, "T-CORE-000107"),
            other => panic!("expected missing setting failure, got {:?}", other),
        }
    }

    #[test]
    fn function_subflow_unknown_function_identifier_fails_when_executed() {
        let engine = ExecutionEngine::new();

        let if_node = node(
            1,
            "std::control::if",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                ),
                function_thunk_param(101, "runnable", "std::missing::function", Vec::new()),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![if_node], None, None, false);

        assert_eq!(reason, ExitReason::Failure);
        match signal {
            Signal::Failure(err) => assert_eq!(err.code, "T-CORE-000002"),
            other => panic!("expected function-not-found failure, got {:?}", other),
        }
    }

    #[test]
    fn function_subflow_can_be_forced_as_eager_argument() {
        let engine = ExecutionEngine::new();

        let add_node = node(
            1,
            "std::number::add",
            vec![
                function_thunk_param(
                    100,
                    "lhs",
                    "std::control::value",
                    vec![subflow_setting("value", Some(int_value(40)), false, true)],
                ),
                literal_param(101, "rhs", int_value(2)),
            ],
            None,
        );

        let (signal, reason) = engine.execute_graph("test", 1, vec![add_node], None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), int_value(42));
    }

    #[test]
    fn execution_report_includes_function_identifier_subflow_results() {
        let mut handlers = FunctionStore::default();
        handlers.populate(&[FunctionRegistration::eager(
            "std::test::echo",
            echo_first_arg_handler,
            1,
        )]);
        let engine = ExecutionEngine {
            handlers,
            sub_flow_registry: SubFlowRegistry::new(),
            compiled_flow_cache: Mutex::new(NonZeroUsize::new(DEFAULT_COMPILED_FLOW_CACHE_CAPACITY).map(
                |cap| CompiledFlowCache::new(cap, DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES),
            )),
        };

        let add_node = node(
            1,
            "std::number::add",
            vec![
                function_thunk_param(
                    100,
                    "lhs",
                    "std::test::echo",
                    vec![subflow_setting("value", Some(int_value(20)), false, true)],
                ),
                literal_param(101, "rhs", int_value(2)),
            ],
            None,
        );

        let report = engine.execute_graph_report("test", 1, vec![add_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(expect_success(report.signal), int_value(22));
        assert_eq!(report.node_execution_results.len(), 2);

        let function_result = &report.node_execution_results[0];
        assert_function_result_id(function_result, "std::test::echo");
        assert_eq!(function_result.parameter_results.len(), 1);
        assert_eq!(
            function_result.parameter_results[0].value,
            Some(int_value(20))
        );
        match function_result.result.as_ref() {
            Some(node_execution_result::Result::Success(value)) => {
                assert_eq!(value, &int_value(20));
            }
            other => panic!("expected function success result, got {:?}", other),
        }

        let node_result = &report.node_execution_results[1];
        assert_node_result_id(node_result, 1);
        match node_result.result.as_ref() {
            Some(node_execution_result::Result::Success(value)) => {
                assert_eq!(value, &int_value(22));
            }
            other => panic!("expected node success result, got {:?}", other),
        }
    }

    #[test]
    fn execution_report_includes_literal_node_parameter_results() {
        let engine = ExecutionEngine::new();
        let add_node = node(
            1,
            "std::number::add",
            vec![
                literal_param(100, "lhs", int_value(1)),
                literal_param(101, "rhs", int_value(2)),
            ],
            None,
        );

        let report = engine.execute_graph_report("test", 1, vec![add_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(report.node_execution_results.len(), 1);

        let node_result = &report.node_execution_results[0];
        assert_node_result_id(node_result, 1);
        assert_eq!(node_result.parameter_results.len(), 2);
        assert_eq!(node_result.parameter_results[0].value, Some(int_value(1)));
        assert_eq!(node_result.parameter_results[1].value, Some(int_value(2)));

        match node_result.result.as_ref() {
            Some(node_execution_result::Result::Success(value)) => {
                assert_eq!(value, &int_value(3));
            }
            other => panic!("expected node success result, got {:?}", other),
        }
    }

    #[test]
    fn execution_report_includes_reference_node_parameter_results() {
        let engine = ExecutionEngine::new();
        let value_node = node(
            1,
            "std::control::value",
            vec![literal_param(100, "value", int_value(7))],
            Some(2),
        );
        let add_node = node(
            2,
            "std::number::add",
            vec![
                node_result_ref_param(200, "lhs", 1),
                literal_param(201, "rhs", int_value(5)),
            ],
            None,
        );

        let report =
            engine.execute_graph_report("test", 1, vec![value_node, add_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(report.node_execution_results.len(), 2);

        let node_result = &report.node_execution_results[1];
        assert_node_result_id(node_result, 2);
        assert_eq!(node_result.parameter_results.len(), 2);
        assert_eq!(node_result.parameter_results[0].value, Some(int_value(7)));
        assert_eq!(node_result.parameter_results[1].value, Some(int_value(5)));

        match node_result.result.as_ref() {
            Some(node_execution_result::Result::Success(value)) => {
                assert_eq!(value, &int_value(12));
            }
            other => panic!("expected node success result, got {:?}", other),
        }
    }

    #[test]
    fn remote_execution_report_converts_missing_outcome_to_node_error() {
        let engine = ExecutionEngine::new();
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(99)),
                result: None,
            },
            target_services: None,
            project_ids: None,
            requests: None,
        };
        let mut remote_node = node(
            1,
            "remote::missing_outcome",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("remote-service".to_string());

        let report =
            engine.execute_graph_report("test", 1, vec![remote_node], None, Some(&remote), false);

        assert_eq!(report.exit_reason, ExitReason::Failure);
        match report.signal {
            Signal::Failure(err) => assert_eq!(err.code, "T-CORE-000006"),
            other => panic!("expected missing-outcome failure, got {:?}", other),
        }
        assert_eq!(report.node_execution_results.len(), 1);

        let node_result = &report.node_execution_results[0];
        assert_node_result_id(node_result, 1);
        assert_eq!(node_result.parameter_results.len(), 1);
        assert_eq!(node_result.parameter_results[0].value, Some(int_value(20)));
        match node_result.result.as_ref() {
            Some(node_execution_result::Result::Error(error)) => {
                assert_eq!(error.code, "T-CORE-000006");
                assert_eq!(error.category, "NodeExecutionResultMissingOutcome");
            }
            other => panic!("expected node error result, got {:?}", other),
        }
    }

    #[test]
    fn remote_execution_strips_action_prefix_from_definition_source() {
        let engine = ExecutionEngine::new();
        let target_services = Arc::new(Mutex::new(Vec::new()));
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(99)),
                result: Some(node_execution_result::Result::Success(string_value("ok"))),
            },
            target_services: Some(Arc::clone(&target_services)),
            project_ids: None,
            requests: None,
        };
        let mut remote_node = node(
            1,
            "remote::stripped_service",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("action.example".to_string());

        let report =
            engine.execute_graph_report("test", 1, vec![remote_node], None, Some(&remote), false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(
            *target_services
                .lock()
                .expect("target service recorder should not be poisoned"),
            vec!["example".to_string()]
        );
    }

    #[test]
    fn remote_execution_uses_flow_project_id() {
        let engine = ExecutionEngine::new();
        let project_ids = Arc::new(Mutex::new(Vec::new()));
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(99)),
                result: Some(node_execution_result::Result::Success(string_value("ok"))),
            },
            target_services: None,
            project_ids: Some(Arc::clone(&project_ids)),
            requests: None,
        };
        let mut remote_node = node(
            1,
            "remote::project",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("action.example".to_string());
        let flow = ExecutionFlow {
            flow_id: 10,
            project_id: 42,
            starting_node_id: 1,
            node_functions: vec![remote_node],
            input_value: None,
        };

        let report = engine.execute_flow_report("test", flow, Some(&remote), false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(
            *project_ids
                .lock()
                .expect("project id recorder should not be poisoned"),
            vec![42]
        );
    }

    /// Records the outgoing request and, while the (mocked) remote call is
    /// still "in flight", probes the sub-flow registry directly through a
    /// handle cloned from the engine before the run started -- proving the
    /// entry exists *during* the call, not just inferring it from the
    /// request shape.
    struct SubFlowMintProbeRuntime {
        registry: SubFlowRegistry,
        result: NodeExecutionResult,
        requests: Arc<Mutex<Vec<ActionExecutionRequest>>>,
        found_pending_during_call: Arc<Mutex<Option<bool>>>,
    }

    #[async_trait]
    impl RemoteRuntime for SubFlowMintProbeRuntime {
        async fn execute_remote(
            &self,
            execution: RemoteExecution,
        ) -> Result<NodeExecutionResult, crate::types::errors::runtime_error::RuntimeError>
        {
            if let Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                execution_identifier,
                input_schema: _,
                output_schema: _,
            })) = &execution.request.parameters[0].value
            {
                *self
                    .found_pending_during_call
                    .lock()
                    .expect("probe recorder should not be poisoned") =
                    Some(self.registry.get(execution_identifier).is_some());
            }
            self.requests
                .lock()
                .expect("request recorder should not be poisoned")
                .push(execution.request.clone());
            Ok(self.result.clone())
        }
    }

    #[test]
    fn remote_node_with_sub_flow_parameter_mints_uuid_instead_of_executing_eagerly() {
        let engine = ExecutionEngine::new();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let found_pending_during_call = Arc::new(Mutex::new(None));
        let remote = SubFlowMintProbeRuntime {
            registry: engine.sub_flow_registry.clone(),
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(1)),
                result: Some(node_execution_result::Result::Success(int_value(1))),
            },
            requests: Arc::clone(&requests),
            found_pending_during_call: Arc::clone(&found_pending_during_call),
        };

        // Node 1 is dispatched remotely and takes node 2's range as a
        // sub-flow-valued parameter instead of a literal.
        let mut remote_node = node(
            1,
            "remote::open_stream",
            vec![thunk_param(100, "on_message", 2)],
            None,
        );
        remote_node.definition_source = Some("action.example".to_string());

        // Node 2 is never actually run by this test -- only the compile +
        // mint + request-shape behavior is under test here.
        let sub_flow_target = node(
            2,
            "std::control::value",
            vec![literal_param(200, "value", int_value(9))],
            None,
        );

        let flow = ExecutionFlow {
            flow_id: 10,
            project_id: 42,
            starting_node_id: 1,
            node_functions: vec![remote_node, sub_flow_target],
            input_value: None,
        };

        let report = engine.execute_flow_report("test", flow, Some(&remote), false);
        assert_eq!(report.exit_reason, ExitReason::Success);

        assert_eq!(
            *found_pending_during_call
                .lock()
                .expect("probe recorder should not be poisoned"),
            Some(true),
            "registry should hold a matching entry while the remote call is outstanding"
        );

        let requests = requests
            .lock()
            .expect("request recorder should not be poisoned");
        assert_eq!(requests.len(), 1);
        let parameters = &requests[0].parameters;
        assert_eq!(parameters.len(), 1);

        let execution_identifier = match &parameters[0].value {
            Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                execution_identifier,
                input_schema: _,
                output_schema: _,
            })) => {
                assert!(
                    uuid::Uuid::parse_str(execution_identifier).is_ok(),
                    "expected a minted UUID, got {:?}",
                    execution_identifier
                );
                execution_identifier.clone()
            }
            other => panic!(
                "expected a minted sub_flow parameter, got {:?} -- the parameter must not be \
                 eagerly resolved to a literal for a remote node",
                other
            ),
        };

        // The parent node's own remote call has resolved (successfully), so
        // the registry entry it minted must already have been cleaned up.
        assert!(
            engine
                .sub_flow_registry
                .get(&execution_identifier)
                .is_none(),
            "registry entry should be removed once the parent call resolves"
        );
    }

    /// Records every outgoing `execution_identifier`. Blocks on `release`
    /// only for the parent's own call (identified by its `SubFlow`-valued
    /// parameter) so the test can drive `execute_sub_flow` while that call
    /// is still outstanding -- exactly the condition under which a nested
    /// remote call inside the sub-flow's own node range would collide with
    /// the parent's still-registered entry in aquila's `PendingReplyStore`
    /// if it reused the parent's `execution_identifier`.
    struct NestedRemoteCapturingRuntime {
        result: NodeExecutionResult,
        minted_sub_flow_id: Arc<Mutex<Option<String>>>,
        release: Arc<tokio::sync::Notify>,
        execution_ids: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl RemoteRuntime for NestedRemoteCapturingRuntime {
        async fn execute_remote(
            &self,
            execution: RemoteExecution,
        ) -> Result<NodeExecutionResult, crate::types::errors::runtime_error::RuntimeError>
        {
            self.execution_ids
                .lock()
                .expect("execution id recorder should not be poisoned")
                .push(execution.request.execution_identifier.clone());

            if let Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
                execution_identifier,
                input_schema: _,
                output_schema: _,
            })) = execution
                .request
                .parameters
                .first()
                .and_then(|param| param.value.as_ref())
            {
                *self
                    .minted_sub_flow_id
                    .lock()
                    .expect("mint recorder should not be poisoned") =
                    Some(execution_identifier.clone());
                // Stay outstanding, mirroring the parent's remote call
                // staying open while sub-flow traffic happens.
                self.release.notified().await;
            }

            Ok(self.result.clone())
        }
    }

    #[test]
    fn sub_flow_execution_uses_a_fresh_execution_identifier_for_its_own_remote_calls() {
        let engine = ExecutionEngine::new();
        let minted_sub_flow_id = Arc::new(Mutex::new(None));
        let release = Arc::new(tokio::sync::Notify::new());
        let execution_ids = Arc::new(Mutex::new(Vec::new()));
        let remote = NestedRemoteCapturingRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(1)),
                result: Some(node_execution_result::Result::Success(int_value(1))),
            },
            minted_sub_flow_id: Arc::clone(&minted_sub_flow_id),
            release: Arc::clone(&release),
            execution_ids: Arc::clone(&execution_ids),
        };

        // Node 1: dispatched remotely (the "parent" call, analogous to
        // `open_stream`), takes node 2's range as a sub-flow parameter.
        let mut remote_node = node(
            1,
            "remote::open_stream",
            vec![thunk_param(100, "on_message", 2)],
            None,
        );
        remote_node.definition_source = Some("action.svc".to_string());

        // Node 2: the sub-flow's own body -- itself dispatched remotely,
        // exactly the scenario in question: a remote call made *from
        // within* a sub-flow's node range while the parent call is open.
        let mut sub_flow_target = node(
            2,
            "remote::callback",
            vec![literal_param(200, "value", int_value(5))],
            None,
        );
        sub_flow_target.definition_source = Some("action.svc".to_string());

        let flow = ExecutionFlow {
            flow_id: 1,
            project_id: 1,
            starting_node_id: 1,
            node_functions: vec![remote_node, sub_flow_target],
            input_value: None,
        };

        std::thread::scope(|scope| {
            scope.spawn(|| {
                let report = engine.execute_flow_report("parent-id", flow, Some(&remote), false);
                assert_eq!(report.exit_reason, ExitReason::Success);
            });

            // Wait for the parent's remote call to mint and capture the
            // sub-flow id, proving the parent call is genuinely still
            // outstanding at this point.
            let sub_flow_execution_id = loop {
                if let Some(id) = minted_sub_flow_id
                    .lock()
                    .expect("mint recorder should not be poisoned")
                    .clone()
                {
                    break id;
                }
                std::thread::sleep(Duration::from_millis(1));
            };

            // Drive the sub-flow's node range while node 1's own call
            // (execution_identifier = "parent-id") is still blocked on
            // `release` -- node 2 being remote means this issues a second,
            // concurrently-outstanding `ActionExecutionRequest`.
            let sub_report = futures_lite::future::block_on(engine.execute_sub_flow(
                &sub_flow_execution_id,
                vec![int_value(7)],
                Some(&remote),
                false,
            ))
            .expect("registry should still have the entry while the parent call is outstanding");
            assert_eq!(sub_report.exit_reason, ExitReason::Success);

            release.notify_one();
        });

        let ids = execution_ids
            .lock()
            .expect("execution id recorder should not be poisoned")
            .clone();
        assert_eq!(ids.len(), 2, "expected exactly one call per remote node");
        assert_eq!(ids[0], "parent-id");
        assert_ne!(
            ids[1], "parent-id",
            "the sub-flow's own remote call must not reuse the parent's execution_identifier -- \
             doing so would collide with the parent's still-outstanding entry in aquila's \
             PendingReplyStore"
        );
        assert!(
            uuid::Uuid::parse_str(&ids[1]).is_ok(),
            "expected a freshly minted run id, got {:?}",
            ids[1]
        );
    }

    #[test]
    fn remote_execution_rejects_empty_action_definition_source() {
        let engine = ExecutionEngine::new();
        let mut remote_node = node(
            1,
            "remote::empty_service",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("action.".to_string());

        let report = engine.execute_graph_report("test", 1, vec![remote_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Failure);
        assert!(report.node_execution_results.is_empty());
        match report.signal {
            Signal::Failure(err) => {
                assert_eq!(err.code, "T-CORE-000106");
                assert_eq!(err.category, "FlowCompileError");
            }
            other => panic!(
                "expected invalid definition_source failure, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn node_execution_result_tracks_actual_node_duration() {
        let mut handlers = FunctionStore::new();
        handlers.populate(&[FunctionRegistration::eager("test::sleep", sleep_handler, 0)]);
        let engine = ExecutionEngine {
            handlers,
            sub_flow_registry: SubFlowRegistry::new(),
            compiled_flow_cache: Mutex::new(NonZeroUsize::new(DEFAULT_COMPILED_FLOW_CACHE_CAPACITY).map(
                |cap| CompiledFlowCache::new(cap, DEFAULT_COMPILED_FLOW_CACHE_MAX_BYTES),
            )),
        };
        let sleep_node = node(1, "test::sleep", vec![], None);

        let report = engine.execute_graph_report("test", 1, vec![sleep_node], None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(report.node_execution_results.len(), 1);

        let node_result = &report.node_execution_results[0];
        assert_node_result_id(node_result, 1);
        assert!(node_result.started_at >= 1_000_000_000_000_000);
        assert!(node_result.finished_at > node_result.started_at);
        assert!(node_result.finished_at - node_result.started_at >= 1_000);
    }

    #[test]
    fn execution_report_keeps_every_for_each_callback_execution() {
        let engine = ExecutionEngine::new();
        let for_each_node = node(
            1,
            "std::list::for_each",
            vec![
                literal_param(
                    100,
                    "list",
                    list_value(vec![int_value(1), int_value(2), int_value(3)]),
                ),
                thunk_param(101, "consumer", 2),
            ],
            None,
        );
        let callback_node = node(
            2,
            "std::number::add",
            vec![
                input_type_ref_param(200, "first", 1, 1, 0),
                literal_param(201, "second", int_value(2)),
            ],
            None,
        );

        let report = engine.execute_graph_report(
            "test",
            1,
            vec![for_each_node, callback_node],
            None,
            None,
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(report.node_execution_results.len(), 4);

        let callback_results: Vec<_> = report
            .node_execution_results
            .iter()
            .filter(|result| result.id == Some(node_execution_result::Id::NodeId(2)))
            .collect();
        assert_eq!(callback_results.len(), 3);

        let callback_values: Vec<_> = callback_results
            .iter()
            .map(|result| match result.result.as_ref() {
                Some(node_execution_result::Result::Success(value)) => value.clone(),
                other => panic!("expected callback success result, got {:?}", other),
            })
            .collect();

        assert_eq!(
            callback_values,
            vec![int_value(3), int_value(4), int_value(5)]
        );
        let callback_parameters: Vec<_> = callback_results
            .iter()
            .map(|result| {
                result
                    .parameter_results
                    .iter()
                    .map(|parameter| parameter.value.clone())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            callback_parameters,
            vec![
                vec![Some(int_value(1)), Some(int_value(2))],
                vec![Some(int_value(2)), Some(int_value(2))],
                vec![Some(int_value(3)), Some(int_value(2))],
            ]
        );
        assert_node_result_id(&report.node_execution_results[3], 1);
    }

    #[test]
    fn execution_report_keeps_every_for_each_function_identifier_callback_execution() {
        let engine = ExecutionEngine::new();
        let response_value = {
            let mut fields = std::collections::HashMap::new();
            fields.insert("http_status_code".to_string(), int_value(200));
            fields.insert("headers".to_string(), empty_struct_value());
            fields.insert("payload".to_string(), string_value("20"));
            fields.insert("http_schema".to_string(), string_value("text/plain"));
            Value {
                kind: Some(Kind::StructValue(Struct { fields })),
            }
        };
        let value_node = node(
            1,
            "std::control::value",
            vec![literal_param(1, "value", response_value.clone())],
            None,
        );
        let mut for_each_node = node(
            2,
            "std::list::for_each",
            vec![
                literal_param(
                    4,
                    "list",
                    list_value(vec![int_value(1), int_value(2), int_value(3)]),
                ),
                function_thunk_param(
                    5,
                    "consumer",
                    "std::boolean::from_number",
                    vec![subflow_setting("value", Some(null_value()), false, false)],
                ),
            ],
            Some(1),
        );
        for_each_node.definition_source = Some("draco-draco-cron".to_string());

        let report = engine.execute_graph_report(
            "test",
            2,
            vec![value_node, for_each_node],
            None,
            None,
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(expect_success(report.signal), response_value);
        assert_eq!(report.node_execution_results.len(), 5);

        let function_results: Vec<_> = report
            .node_execution_results
            .iter()
            .filter(|result| {
                result.id
                    == Some(node_execution_result::Id::FunctionIdentifier(
                        "std::boolean::from_number".to_string(),
                    ))
            })
            .collect();
        assert_eq!(function_results.len(), 3);

        for (index, result) in function_results.iter().enumerate() {
            assert_eq!(result.parameter_results.len(), 1);
            assert_eq!(
                result.parameter_results[0].value,
                Some(int_value(index as i64 + 1))
            );
            match result.result.as_ref() {
                Some(node_execution_result::Result::Success(value)) => {
                    assert_eq!(
                        value,
                        &Value {
                            kind: Some(Kind::BoolValue(true)),
                        }
                    );
                }
                other => panic!("expected function success result, got {:?}", other),
            }
        }

        assert_function_result_id(
            &report.node_execution_results[0],
            "std::boolean::from_number",
        );
        assert_function_result_id(
            &report.node_execution_results[1],
            "std::boolean::from_number",
        );
        assert_function_result_id(
            &report.node_execution_results[2],
            "std::boolean::from_number",
        );
        assert_node_result_id(&report.node_execution_results[3], 2);
        assert_node_result_id(&report.node_execution_results[4], 1);
    }

    /// Proves the compiled-flow cache actually short-circuits recompilation
    /// (not just "still works") by asserting the second execution's cached
    /// `Arc<CompiledFlow>` is the *same allocation* as the first, rather than
    /// timing anything -- a wall-clock assertion here would be flaky under
    /// CI load. Performance numbers live in `taurus-bench` (criterion),
    /// where noise is handled statistically instead of by a hand assertion.
    #[test]
    fn compiled_flow_cache_reuses_arc_across_executions_of_the_same_flow() {
        let engine = ExecutionEngine::new();
        let add_node = node(
            1,
            "std::number::add",
            vec![
                literal_param(0, "a", int_value(1)),
                literal_param(0, "b", int_value(2)),
            ],
            None,
        );
        let nodes = vec![add_node];
        let key = compiled_flow_cache_key(0, 1, &nodes);

        let (signal, reason) =
            engine.execute_graph("run-1", 1, nodes.clone(), None, None, false);
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), int_value(3));
        let first = engine
            .compiled_flow_cache
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|cache| cache.peek(&key))
            .expect("flow should be cached after first execution");

        let (signal, reason) = engine.execute_graph("run-2", 1, nodes, None, None, false);
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), int_value(3));
        let second = engine
            .compiled_flow_cache
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|cache| cache.peek(&key))
            .expect("flow should still be cached after second execution");

        assert!(
            Arc::ptr_eq(&first, &second),
            "second execution should reuse the cached Arc<CompiledFlow>, not recompile"
        );
    }

    /// A structurally different flow (different node id / handler) must not
    /// collide with an unrelated cached entry.
    #[test]
    fn compiled_flow_cache_key_differs_for_different_flows() {
        let nodes_a = vec![node(
            1,
            "std::number::add",
            vec![
                literal_param(0, "a", int_value(1)),
                literal_param(0, "b", int_value(2)),
            ],
            None,
        )];
        let nodes_b = vec![node(
            1,
            "std::number::add",
            vec![
                literal_param(0, "a", int_value(1)),
                literal_param(0, "b", int_value(99)),
            ],
            None,
        )];

        assert_ne!(
            compiled_flow_cache_key(0, 1, &nodes_a),
            compiled_flow_cache_key(0, 1, &nodes_b)
        );
    }

    /// Capacity 0 must disable the cache: nothing is ever stored, so every
    /// execution recompiles, matching pre-cache behavior exactly.
    #[test]
    fn compiled_flow_cache_capacity_zero_disables_caching() {
        let engine = ExecutionEngine::with_compiled_flow_cache_capacity(0);
        let nodes = vec![node(
            1,
            "std::number::add",
            vec![
                literal_param(0, "a", int_value(1)),
                literal_param(0, "b", int_value(2)),
            ],
            None,
        )];

        let (signal, reason) =
            engine.execute_graph("run-1", 1, nodes.clone(), None, None, false);
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), int_value(3));

        assert!(
            engine.compiled_flow_cache.lock().unwrap().is_none(),
            "capacity 0 should leave the cache disabled (None), never populated"
        );
    }

    /// A byte budget too small for every flow must evict the
    /// least-recently-used entry, not just refuse new inserts -- proves the
    /// cache is self-bounding by size, not only by entry count (the entry
    /// count alone can't prevent unbounded memory growth if individual
    /// flows are multi-MB).
    #[test]
    fn compiled_flow_cache_evicts_lru_entry_when_byte_budget_exceeded() {
        fn add_node(b: i64) -> NodeFunction {
            node(
                1,
                "std::number::add",
                vec![
                    literal_param(0, "a", int_value(1)),
                    literal_param(0, "b", int_value(b)),
                ],
                None,
            )
        }

        let flow_a = vec![add_node(2)];
        let flow_b = vec![add_node(3)];
        let flow_c = vec![add_node(4)];

        let (key_a, weight_a) = compiled_flow_cache_key_and_weight(0, 1, &flow_a);
        let (key_b, weight_b) = compiled_flow_cache_key_and_weight(0, 1, &flow_b);
        let (key_c, weight_c) = compiled_flow_cache_key_and_weight(0, 1, &flow_c);
        assert_eq!(
            weight_a, weight_b,
            "structurally identical flows should weigh the same"
        );
        assert_eq!(
            weight_a, weight_c,
            "structurally identical flows should weigh the same"
        );

        // Room for exactly two entries; a generous entry-count cap so only
        // the byte budget is actually under test here.
        let max_bytes = weight_a + weight_b;
        let engine = ExecutionEngine::with_compiled_flow_cache_limits(100, max_bytes);

        let _ = engine.execute_graph("a", 1, flow_a, None, None, false);
        let _ = engine.execute_graph("b", 1, flow_b, None, None, false);
        {
            let cache = engine.compiled_flow_cache.lock().unwrap();
            assert_eq!(cache.as_ref().unwrap().len(), 2);
        }

        // A third distinct flow pushes total weight past the budget. `a`
        // is the least-recently-used entry (never touched since its own
        // insert) and should be the one evicted, not `b`.
        let _ = engine.execute_graph("c", 1, flow_c, None, None, false);

        let cache = engine.compiled_flow_cache.lock().unwrap();
        let cache = cache.as_ref().unwrap();
        assert!(
            cache.peek(&key_a).is_none(),
            "least-recently-used entry should have been evicted"
        );
        assert!(
            cache.peek(&key_b).is_some(),
            "more recently used entry should survive"
        );
        assert!(
            cache.peek(&key_c).is_some(),
            "newly inserted entry should be present"
        );
        assert_eq!(cache.len(), 2);
    }

    /// A single flow bigger than the entire byte budget must not be cached
    /// at all -- caching it would just evict everything else (including
    /// itself, on the very next insert), so it's simplest to let it always
    /// recompile instead of thrashing the cache.
    #[test]
    fn compiled_flow_cache_skips_a_single_flow_larger_than_the_whole_budget() {
        let nodes = vec![node(
            1,
            "std::number::add",
            vec![
                literal_param(0, "a", int_value(1)),
                literal_param(0, "b", int_value(2)),
            ],
            None,
        )];
        let (key, weight) = compiled_flow_cache_key_and_weight(0, 1, &nodes);

        let engine = ExecutionEngine::with_compiled_flow_cache_limits(100, weight - 1);
        let (signal, reason) = engine.execute_graph("run", 1, nodes, None, None, false);
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(expect_success(signal), int_value(3));

        let cache = engine.compiled_flow_cache.lock().unwrap();
        assert!(
            cache.as_ref().unwrap().peek(&key).is_none(),
            "a flow bigger than the whole budget should not be cached"
        );
    }

    /// `if`'s `runnable` branch re-enters the executor through the
    /// synchronous thunk path (`execute_from_index_sync`). A `Remote`
    /// node inside that branch used to hard-fail with
    /// `RemoteRuntimeRequiresAsyncExecution` -- it now bridges through
    /// `block_on`, the same pattern already used for a local
    /// function-thunk's remote call.
    #[test]
    fn if_branch_can_execute_a_remote_node() {
        let engine = ExecutionEngine::new();
        let target_services = Arc::new(Mutex::new(Vec::new()));
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(2)),
                result: Some(node_execution_result::Result::Success(int_value(42))),
            },
            target_services: Some(Arc::clone(&target_services)),
            project_ids: None,
            requests: None,
        };

        let if_node = node(
            1,
            "std::control::if",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                ),
                thunk_param(101, "runnable", 2),
            ],
            None,
        );
        let mut remote_branch_node = node(
            2,
            "remote::branch_add",
            vec![literal_param(200, "payload", int_value(1))],
            None,
        );
        remote_branch_node.definition_source = Some("action.example".to_string());

        let report = engine.execute_graph_report(
            "test",
            1,
            vec![if_node, remote_branch_node],
            None,
            Some(&remote),
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(expect_success(report.signal), int_value(42));
        assert_eq!(
            *target_services
                .lock()
                .expect("target service recorder should not be poisoned"),
            vec!["example".to_string()]
        );
    }

    /// Same as above but for `if_else`'s `else_runnable` branch, to prove
    /// the fix isn't `if`-specific (both share the same sync thunk path).
    #[test]
    fn if_else_branch_can_execute_a_remote_node() {
        let engine = ExecutionEngine::new();
        let remote = StubRemoteRuntime {
            result: NodeExecutionResult {
                started_at: 1,
                finished_at: 2,
                parameter_results: Vec::new(),
                id: Some(node_execution_result::Id::NodeId(3)),
                result: Some(node_execution_result::Result::Success(int_value(7))),
            },
            target_services: None,
            project_ids: None,
            requests: None,
        };

        let if_else_node = node(
            1,
            "std::control::if_else",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(false)),
                    },
                ),
                thunk_param(101, "runnable", 2),
                thunk_param(102, "else_runnable", 3),
            ],
            None,
        );
        let then_branch_node = node(
            2,
            "std::control::value",
            vec![literal_param(200, "value", int_value(999))],
            None,
        );
        let mut else_branch_node = node(
            3,
            "remote::branch_add",
            vec![literal_param(300, "payload", int_value(1))],
            None,
        );
        else_branch_node.definition_source = Some("action.example".to_string());

        let report = engine.execute_graph_report(
            "test",
            1,
            vec![if_else_node, then_branch_node, else_branch_node],
            None,
            Some(&remote),
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(expect_success(report.signal), int_value(7));
    }

    /// `stop` used to vanish entirely from `node_execution_results`
    /// (`commit_result`'s `other => other` branch skipped recording any
    /// non-Success/Failure signal). It's now recorded as `Success(null)`
    /// -- while execution still halts exactly as before, proven here by
    /// asserting node 2 never runs.
    #[test]
    fn stop_node_is_recorded_as_success_and_still_halts_execution() {
        let engine = ExecutionEngine::new();
        let stop_node = node(1, "std::control::stop", vec![], Some(2));
        let unreachable_node = node(
            2,
            "std::control::value",
            vec![literal_param(100, "value", int_value(99))],
            None,
        );

        let report = engine.execute_graph_report(
            "test",
            1,
            vec![stop_node, unreachable_node],
            None,
            None,
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Stop);
        assert_eq!(report.node_execution_results.len(), 1);
        assert_eq!(
            report.node_execution_results[0].id,
            Some(node_execution_result::Id::NodeId(1))
        );
        assert_eq!(
            report.node_execution_results[0].result,
            Some(node_execution_result::Result::Success(Value {
                kind: Some(Kind::NullValue(0)),
            }))
        );
    }

    /// Same root cause, one level up: `if`'s handler tail-returns whatever
    /// its branch returns, so a branch calling `stop` used to make `if`'s
    /// *own* node result vanish too (it was committing the same
    /// unconverted `Signal::Stop`). Both `if` and `stop` must now show up.
    #[test]
    fn if_wrapping_stop_records_both_if_and_stop_nodes() {
        let engine = ExecutionEngine::new();
        let if_node = node(
            1,
            "std::control::if",
            vec![
                literal_param(
                    100,
                    "condition",
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                ),
                thunk_param(101, "runnable", 2),
            ],
            Some(3),
        );
        let stop_node = node(2, "std::control::stop", vec![], None);
        let unreachable_node = node(
            3,
            "std::control::value",
            vec![literal_param(300, "value", int_value(99))],
            None,
        );

        let report = engine.execute_graph_report(
            "test",
            1,
            vec![if_node, stop_node, unreachable_node],
            None,
            None,
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Stop);
        let recorded_ids: Vec<_> = report
            .node_execution_results
            .iter()
            .map(|result| result.id.clone())
            .collect();
        assert_eq!(
            recorded_ids,
            vec![
                Some(node_execution_result::Id::NodeId(2)),
                Some(node_execution_result::Id::NodeId(1)),
            ]
        );
        for result in &report.node_execution_results {
            assert_eq!(
                result.result,
                Some(node_execution_result::Result::Success(Value {
                    kind: Some(Kind::NullValue(0)),
                }))
            );
        }
    }
}
