//! Public runtime execution API.
//!
//! This module is the new entrypoint for flow execution from external crates.
//! It executes compiled flow plans via the runtime engine executor loop.

mod compiler;
mod emitter;
mod executor;
mod model;

use tucana::shared::{ExecutionFlow, NodeFunction, Value};

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
        self.execute_flow_with_execution_id(
            ExecutionId::new_v4(),
            flow,
            remote,
            respond_emitter,
            with_trace,
        )
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
        self.execute_graph_with_execution_id(
            execution_id,
            flow.starting_node_id,
            flow.node_functions,
            flow.input_value,
            remote,
            respond_emitter,
            with_trace,
        )
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
        self.execute_graph_with_execution_id(
            ExecutionId::new_v4(),
            start_node_id,
            node_functions,
            flow_input,
            remote,
            respond_emitter,
            with_trace,
        )
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
                return (signal, ExitReason::Failure);
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
        );
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
        (signal, exit_reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::exit_reason::ExitReason;
    use std::cell::RefCell;
    use tucana::shared::{
        InputType, ListValue, NodeParameter, NodeValue, ReferenceValue, Struct, Value, node_value,
        reference_value, value::Kind,
    };

    fn literal_param(database_id: i64, runtime_parameter_id: &str, value: Value) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::LiteralValue(value)),
            }),
        }
    }

    fn thunk_param(database_id: i64, runtime_parameter_id: &str, node_id: i64) -> NodeParameter {
        NodeParameter {
            database_id,
            runtime_parameter_id: runtime_parameter_id.to_string(),
            value: Some(NodeValue {
                value: Some(node_value::Value::NodeFunctionId(node_id)),
            }),
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
        }
    }

    fn node(
        database_id: i64,
        runtime_function_id: &str,
        parameters: Vec<NodeParameter>,
        next_node_id: Option<i64>,
    ) -> NodeFunction {
        NodeFunction {
            database_id,
            runtime_function_id: runtime_function_id.to_string(),
            parameters,
            next_node_id,
            definition_source: "taurus".to_string(),
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
    fn emitter_emits_start_and_finish_for_successful_execution() {
        let engine = ExecutionEngine::new();
        let events = RefCell::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events.borrow_mut().push(emit_type);
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
            *events.borrow(),
            vec![EmitType::StartingExec, EmitType::FinishedExec]
        );
    }

    #[test]
    fn emitter_emits_ongoing_for_intermediate_respond() {
        let engine = ExecutionEngine::new();
        let events = RefCell::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events.borrow_mut().push(emit_type);
        };

        let create_response_node = node(
            1,
            "http::response::create",
            vec![
                literal_param(100, "http_status_code", int_value(200)),
                literal_param(101, "headers", empty_struct_value()),
                literal_param(102, "payload", string_value("hello")),
            ],
            Some(2),
        );
        let respond_node = node(
            2,
            "rest::control::respond",
            vec![node_result_ref_param(200, "response", 1)],
            Some(3),
        );
        let finish_node = node(
            3,
            "std::number::add",
            vec![
                literal_param(300, "lhs", int_value(1)),
                literal_param(301, "rhs", int_value(1)),
            ],
            None,
        );

        let (_signal, reason) = engine.execute_graph(
            1,
            vec![create_response_node, respond_node, finish_node],
            None,
            None,
            Some(&emitter),
            false,
        );
        assert_eq!(reason, ExitReason::Success);
        assert_eq!(
            *events.borrow(),
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
        let events = RefCell::new(Vec::<EmitType>::new());
        let emitter = |_execution_id, emit_type: EmitType, _value: Value| {
            events.borrow_mut().push(emit_type);
        };

        let invalid_add_node = node(1, "std::number::add", vec![], None);

        let (_signal, reason) =
            engine.execute_graph(1, vec![invalid_add_node], None, None, Some(&emitter), false);
        assert_eq!(reason, ExitReason::Failure);
        assert_eq!(
            *events.borrow(),
            vec![EmitType::StartingExec, EmitType::FailedExec]
        );
    }
}
