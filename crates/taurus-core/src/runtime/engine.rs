//! Public runtime execution API.
//!
//! This module is the new entrypoint for flow execution from external crates.
//! It executes compiled flow plans via the runtime engine executor loop.

mod compiler;
mod emitter;
mod executor;
mod model;

use futures_lite::future::block_on;
use tucana::shared::{ExecutionFlow, NodeExecutionResult, NodeFunction, Value};

use crate::handler::registry::FunctionStore;
use crate::runtime::execution::value_store::ValueStore;
use crate::runtime::remote::RemoteRuntime;
use crate::types::exit_reason::ExitReason;
use crate::types::signal::Signal;
use compiler::compile_flow;
pub use emitter::{EmitType, ExecutionId, RespondEmitter};

fn null_value() -> Value {
    Value {
        kind: Some(tucana::shared::value::Kind::NullValue(0)),
    }
}

/// Runtime engine entrypoint used by runtime binaries and CLI tools.
pub struct ExecutionEngine {
    handlers: FunctionStore,
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
    /// Build a new execution engine with default handler registry.
    pub fn new() -> Self {
        Self {
            handlers: FunctionStore::default(),
        }
    }

    /// Execute an `ExecutionFlow`.
    pub fn execute_flow(
        &self,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = block_on(self.execute_flow_with_execution_id_report_async(
            ExecutionId::new_v4(),
            flow,
            remote,
            respond_emitter,
            with_trace,
        ));
        (report.signal, report.exit_reason)
    }

    /// Execute an `ExecutionFlow` asynchronously.
    pub async fn execute_flow_async(
        &self,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = self
            .execute_flow_with_execution_id_report_async(
                ExecutionId::new_v4(),
                flow,
                remote,
                respond_emitter,
                with_trace,
            )
            .await;
        (report.signal, report.exit_reason)
    }

    /// Execute an `ExecutionFlow` with a caller-provided execution id.
    pub fn execute_flow_with_execution_id(
        &self,
        execution_id: ExecutionId,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = block_on(self.execute_flow_with_execution_id_report_async(
            execution_id,
            flow,
            remote,
            respond_emitter,
            with_trace,
        ));
        (report.signal, report.exit_reason)
    }

    /// Execute an `ExecutionFlow` asynchronously with a caller-provided execution id.
    pub async fn execute_flow_with_execution_id_async(
        &self,
        execution_id: ExecutionId,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = self
            .execute_flow_with_execution_id_report_async(
                execution_id,
                flow,
                remote,
                respond_emitter,
                with_trace,
            )
            .await;
        (report.signal, report.exit_reason)
    }

    /// Execute an `ExecutionFlow` and return the final signal plus per-node execution results.
    pub fn execute_flow_report(
        &self,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_flow_with_execution_id_report_async(
            ExecutionId::new_v4(),
            flow,
            remote,
            respond_emitter,
            with_trace,
        ))
    }

    /// Execute an `ExecutionFlow` asynchronously and return per-node execution results.
    pub async fn execute_flow_report_async(
        &self,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        self.execute_flow_with_execution_id_report_async(
            ExecutionId::new_v4(),
            flow,
            remote,
            respond_emitter,
            with_trace,
        )
        .await
    }

    /// Execute an `ExecutionFlow` with a caller-provided execution id and return per-node results.
    pub fn execute_flow_with_execution_id_report(
        &self,
        execution_id: ExecutionId,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_flow_with_execution_id_report_async(
            execution_id,
            flow,
            remote,
            respond_emitter,
            with_trace,
        ))
    }

    /// Execute an `ExecutionFlow` asynchronously with a caller-provided execution id and return per-node results.
    pub async fn execute_flow_with_execution_id_report_async(
        &self,
        execution_id: ExecutionId,
        flow: ExecutionFlow,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        self.execute_graph_with_execution_id_report_async(
            execution_id,
            flow.starting_node_id,
            flow.node_functions,
            flow.input_value,
            remote,
            respond_emitter,
            with_trace,
        )
        .await
    }

    /// Execute a graph described by node list and start node.
    pub fn execute_graph(
        &self,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = block_on(self.execute_graph_with_execution_id_report_async(
            ExecutionId::new_v4(),
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        ));
        (report.signal, report.exit_reason)
    }

    /// Execute a graph asynchronously.
    pub async fn execute_graph_async(
        &self,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = self
            .execute_graph_with_execution_id_report_async(
                ExecutionId::new_v4(),
                start_node_id,
                node_functions,
                flow_input,
                remote,
                respond_emitter,
                with_trace,
            )
            .await;
        (report.signal, report.exit_reason)
    }

    /// Execute a graph described by node list and start node with a caller-provided execution id.
    pub fn execute_graph_with_execution_id(
        &self,
        execution_id: ExecutionId,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = block_on(self.execute_graph_with_execution_id_report_async(
            execution_id,
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        ));
        (report.signal, report.exit_reason)
    }

    /// Execute a graph asynchronously with a caller-provided execution id.
    pub async fn execute_graph_with_execution_id_async(
        &self,
        execution_id: ExecutionId,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> (Signal, ExitReason) {
        let report = self
            .execute_graph_with_execution_id_report_async(
                execution_id,
                start_node_id,
                node_functions,
                flow_input,
                remote,
                respond_emitter,
                with_trace,
            )
            .await;
        (report.signal, report.exit_reason)
    }

    /// Execute a graph and return the final signal plus per-node execution results.
    pub fn execute_graph_report(
        &self,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_graph_with_execution_id_report_async(
            ExecutionId::new_v4(),
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        ))
    }

    /// Execute a graph asynchronously and return per-node execution results.
    pub async fn execute_graph_report_async(
        &self,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        self.execute_graph_with_execution_id_report_async(
            ExecutionId::new_v4(),
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        )
        .await
    }

    /// Execute a graph with a caller-provided execution id and return per-node results.
    pub fn execute_graph_with_execution_id_report(
        &self,
        execution_id: ExecutionId,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        block_on(self.execute_graph_with_execution_id_report_async(
            execution_id,
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        ))
    }

    /// Execute a graph asynchronously with a caller-provided execution id and return per-node results.
    pub async fn execute_graph_with_execution_id_report_async(
        &self,
        execution_id: ExecutionId,
        start_node_id: i64,
        node_functions: Vec<NodeFunction>,
        flow_input: Option<Value>,
        remote: Option<&dyn RemoteRuntime>,
        respond_emitter: Option<&dyn RespondEmitter>,
        with_trace: bool,
    ) -> EngineExecutionReport {
        if let Some(emitter) = respond_emitter {
            emitter.emit(execution_id, EmitType::StartingExec, null_value());
        }

        let mut value_store = match flow_input {
            Some(v) => ValueStore::new(v),
            None => ValueStore::default(),
        };

        let compiled = match compile_flow(start_node_id, node_functions) {
            Ok(plan) => plan,
            Err(err) => {
                let runtime_error = err.as_runtime_error();
                if let Some(emitter) = respond_emitter {
                    emitter.emit(execution_id, EmitType::FailedExec, runtime_error.as_value());
                }
                let signal = Signal::Failure(runtime_error);
                return EngineExecutionReport {
                    signal,
                    exit_reason: ExitReason::Failure,
                    node_execution_results: Vec::new(),
                };
            }
        };

        let (signal, trace_run) = executor::execute_compiled(
            &compiled,
            &self.handlers,
            &mut value_store,
            remote,
            execution_id,
            respond_emitter,
            with_trace,
        )
        .await;
        if with_trace && let Some(trace_run) = trace_run {
            println!(
                "{}",
                crate::runtime::execution::render::render_trace(&trace_run)
            );
        }
        if let Some(emitter) = respond_emitter {
            match &signal {
                Signal::Failure(err) => {
                    emitter.emit(execution_id, EmitType::FailedExec, err.as_value())
                }
                Signal::Success(value) | Signal::Return(value) | Signal::Respond(value) => {
                    emitter.emit(execution_id, EmitType::FinishedExec, value.clone())
                }
                Signal::Stop => emitter.emit(execution_id, EmitType::FinishedExec, null_value()),
            }
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
    use tucana::shared::{
        InputType, ListValue, NodeExecutionResult, NodeParameter, NodeValue, ReferenceValue,
        Struct, SubFlow, SubFlowSetting, Value, node_execution_result, node_value, reference_value,
        sub_flow::ExecutionReference, value::Kind,
    };

    fn literal_param(database_id: i64, runtime_parameter_id: &str, value: Value) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::LiteralValue(value)),
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
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::SubFlow(SubFlow {
                    signature: String::new(),
                    settings,
                    execution_reference: Some(ExecutionReference::FunctionIdentifier(
                        function_identifier.to_string(),
                    )),
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
            2,
            vec![parent, next, return_node, unreachable_after_return],
            None,
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

        let (signal, reason) = engine.execute_graph(1, vec![map_node], None, None, None, false);

        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            expect_success(signal),
            list_value(vec![int_value(3), int_value(4)])
        );
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

        let (signal, reason) = engine.execute_graph(1, vec![filter_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![map_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![map_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![if_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![if_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![if_node], None, None, None, false);

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

        let (signal, reason) = engine.execute_graph(1, vec![add_node], None, None, None, false);

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
        let engine = ExecutionEngine { handlers };

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

        let report = engine.execute_graph_report(1, vec![add_node], None, None, None, false);

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

        let report = engine.execute_graph_report(1, vec![add_node], None, None, None, false);

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
            engine.execute_graph_report(1, vec![value_node, add_node], None, None, None, false);

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
    fn execution_report_includes_respond_node_parameter_results() {
        let engine = ExecutionEngine::new();
        let respond_node = node(
            1,
            "rest::control::respond",
            vec![
                literal_param(100, "http_status_code", int_value(200)),
                literal_param(101, "headers", empty_struct_value()),
                literal_param(102, "http_schema", string_value("application/json")),
                literal_param(103, "payload", string_value("hello")),
            ],
            None,
        );

        let report = engine.execute_graph_report(1, vec![respond_node], None, None, None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(report.node_execution_results.len(), 1);

        let node_result = &report.node_execution_results[0];
        assert_node_result_id(node_result, 1);
        assert_eq!(node_result.parameter_results.len(), 4);
        assert_eq!(node_result.parameter_results[0].value, Some(int_value(200)));
        assert_eq!(
            node_result.parameter_results[1].value,
            Some(empty_struct_value())
        );
        assert_eq!(
            node_result.parameter_results[2].value,
            Some(string_value("application/json"))
        );
        assert_eq!(
            node_result.parameter_results[3].value,
            Some(string_value("hello"))
        );
        assert!(matches!(
            node_result.result,
            Some(node_execution_result::Result::Success(_))
        ));
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
        };
        let mut remote_node = node(
            1,
            "remote::missing_outcome",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("remote-service".to_string());

        let report =
            engine.execute_graph_report(1, vec![remote_node], None, Some(&remote), None, false);

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
        };
        let mut remote_node = node(
            1,
            "remote::stripped_service",
            vec![literal_param(100, "payload", int_value(20))],
            None,
        );
        remote_node.definition_source = Some("action.example".to_string());

        let report =
            engine.execute_graph_report(1, vec![remote_node], None, Some(&remote), None, false);

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(
            *target_services
                .lock()
                .expect("target service recorder should not be poisoned"),
            vec!["example".to_string()]
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

        let report = engine.execute_graph_report(1, vec![remote_node], None, None, None, false);

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
        let engine = ExecutionEngine { handlers };
        let sleep_node = node(1, "test::sleep", vec![], None);

        let report = engine.execute_graph_report(1, vec![sleep_node], None, None, None, false);

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
            1,
            vec![for_each_node, callback_node],
            None,
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
        let mut respond_node = node(
            1,
            "rest::control::respond",
            vec![
                literal_param(1, "http_status_code", int_value(200)),
                literal_param(2, "headers", empty_struct_value()),
                literal_param(3, "http_schema", string_value("text/plain")),
                literal_param(4, "payload", string_value("20")),
            ],
            None,
        );
        respond_node.definition_source = Some("draco-draco-cron".to_string());
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
            2,
            vec![respond_node, for_each_node],
            None,
            None,
            None,
            false,
        );

        assert_eq!(report.exit_reason, ExitReason::Success);
        assert_eq!(expect_success(report.signal), {
            let mut fields = std::collections::HashMap::new();
            fields.insert("http_status_code".to_string(), int_value(200));
            fields.insert("headers".to_string(), empty_struct_value());
            fields.insert("payload".to_string(), string_value("20"));
            Value {
                kind: Some(Kind::StructValue(Struct { fields })),
            }
        });
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

    #[test]
    fn emitter_emits_start_and_finish_for_successful_execution() {
        let engine = ExecutionEngine::new();
        let events = Mutex::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events
                .lock()
                .expect("event recorder should not be poisoned")
                .push(emit_type);
        };

        let add_node = node(
            1,
            "std::number::add",
            vec![
                literal_param(100, "lhs", int_value(1)),
                literal_param(101, "rhs", int_value(2)),
            ],
            None,
        );

        let (_signal, reason) =
            engine.execute_graph(1, vec![add_node], None, None, Some(&emitter), false);
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            *events
                .lock()
                .expect("event recorder should not be poisoned"),
            vec![EmitType::StartingExec, EmitType::FinishedExec]
        );
    }

    #[test]
    fn emitter_emits_ongoing_for_intermediate_respond() {
        let engine = ExecutionEngine::new();
        let events = Mutex::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events
                .lock()
                .expect("event recorder should not be poisoned")
                .push(emit_type);
        };

        let respond_node = node(
            1,
            "rest::control::respond",
            vec![
                literal_param(100, "http_status_code", int_value(200)),
                literal_param(101, "headers", empty_struct_value()),
                literal_param(102, "http_schema", string_value("application/json")),
                literal_param(103, "payload", string_value("hello")),
            ],
            Some(2),
        );
        let finish_node = node(
            2,
            "std::number::add",
            vec![
                literal_param(300, "lhs", int_value(1)),
                literal_param(301, "rhs", int_value(1)),
            ],
            None,
        );

        let (_signal, reason) = engine.execute_graph(
            1,
            vec![respond_node, finish_node],
            None,
            None,
            Some(&emitter),
            false,
        );
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            *events
                .lock()
                .expect("event recorder should not be poisoned"),
            vec![
                EmitType::StartingExec,
                EmitType::OngoingExec,
                EmitType::FinishedExec
            ]
        );
    }

    #[test]
    fn emitter_emits_failed_for_runtime_failure() {
        let engine = ExecutionEngine::new();
        let events = Mutex::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events
                .lock()
                .expect("event recorder should not be poisoned")
                .push(emit_type);
        };

        let invalid_add_node = node(1, "std::number::add", vec![], None);

        let (_signal, reason) =
            engine.execute_graph(1, vec![invalid_add_node], None, None, Some(&emitter), false);
        assert_eq!(reason, ExitReason::Failure);
        assert_eq!(
            *events
                .lock()
                .expect("event recorder should not be poisoned"),
            vec![EmitType::StartingExec, EmitType::FailedExec]
        );
    }
}
