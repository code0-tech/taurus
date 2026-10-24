//! In-process stand-in for the actions flows in the remote suite call, so
//! those fixtures exercise Taurus' real remote paths -- request building,
//! sub-flow minting, action-driven `execute_sub_flow` callbacks, and sub flow
//! functions returned as node results -- without NATS or Aquila.
//!
//! Each function below behaves like a small real action would with its
//! `ActionExecutionRequest`, keyed by the target service and function
//! identifier the flow dispatched to:
//!
//! | Service | Function | Behavior |
//! | --- | --- | --- |
//! | `example` | `remote::identity(value)` | Returns `value`. |
//! | `example` | `remote::double(value)` | Returns `value * 2`. |
//! | `example` | `remote::fail()` | Always fails -- proves a function is never run. |
//! | `example` | `remote::sub_flow_function(function_identifier, definition_source)` | Returns a sub flow function (`SubFlowFunctionResult`) instead of a value. |
//! | `example` | `remote::apply(handler, value)` | Invokes the `handler` sub flow once with `value` and returns its result. |
//! | `example` | `remote::keep(handler)` | Receives a `handler` sub flow but never invokes it; returns `"kept"`. |
//! | `websocket-action` | `for_each(list, consumer)` | Invokes `consumer` once per element; returns `null`. |

use taurus_core::runtime::engine::ExecutionEngine;
use taurus_core::runtime::remote::{RemoteExecution, RemoteRuntime};
use taurus_core::types::errors::runtime_error::RuntimeError;
use taurus_core::types::signal::Signal;
use taurus_core::value::{number_to_f64, raw_value_result, value_from_f64, value_from_i64};
use tucana::aquila::{ActionNodeSubFlowValue, ActionNodeValue, action_node_value};
use tucana::shared::node_execution_result::{
    Id as NodeExecutionResultId, Result as NodeExecutionOutcome,
};
use tucana::shared::value::Kind;
use tucana::shared::{
    NodeExecutionResult, NodeValueExecutionResult, NumberValue, SubFlowFunction,
    SubFlowFunctionResult, Value, node_value_execution_result, number_value,
};

pub struct SimulatedActions<'a> {
    /// The engine the flow itself runs on, so a simulated action invoking a
    /// minted sub flow resolves it against the registry entry the flow's own
    /// remote call created.
    engine: &'a ExecutionEngine,
}

impl<'a> SimulatedActions<'a> {
    pub fn new(engine: &'a ExecutionEngine) -> Self {
        Self { engine }
    }
}

#[async_trait::async_trait]
impl RemoteRuntime for SimulatedActions<'_> {
    async fn execute_remote(
        &self,
        execution: RemoteExecution,
    ) -> Result<NodeExecutionResult, RuntimeError> {
        let parameters = &execution.request.parameters;
        let function_identifier = execution.request.function_identifier.as_str();

        let result = match (execution.target_service.as_str(), function_identifier) {
            ("example", "remote::identity") => raw(literal(parameters, 0)?),
            ("example", "remote::double") => raw(double(&literal(parameters, 0)?)?),
            ("example", "remote::fail") => {
                return Err(RuntimeError::new(
                    "T-TEST-000006",
                    "SimulatedActionFailure",
                    "remote::fail was executed",
                ));
            }
            ("example", "remote::sub_flow_function") => sub_flow_function(parameters)?,
            ("example", "remote::apply") => {
                let handler = sub_flow(parameters, 0)?;
                let value = literal(parameters, 1)?;
                raw(self.invoke(&handler, value).await?)
            }
            ("example", "remote::keep") => {
                sub_flow(parameters, 0)?;
                raw(string("kept"))
            }
            ("websocket-action", "for_each") => {
                let consumer = sub_flow(parameters, 1)?;
                for item in list(&literal(parameters, 0)?)? {
                    self.invoke(&consumer, item).await?;
                }
                raw(null())
            }
            (service, function) => {
                return Err(RuntimeError::new(
                    "T-TEST-000001",
                    "UnknownSimulatedFunction",
                    format!(
                        "No simulated action function {} on service {}",
                        function, service
                    ),
                ));
            }
        };

        Ok(NodeExecutionResult {
            started_at: 0,
            finished_at: 0,
            parameter_results: Vec::new(),
            id: Some(NodeExecutionResultId::FunctionIdentifier(
                function_identifier.to_string(),
            )),
            result: Some(NodeExecutionOutcome::Success(result)),
        })
    }
}

impl SimulatedActions<'_> {
    /// Invokes a minted sub flow the way an action does over
    /// `ActionSubFlowExecutionRequest`, failing the whole remote call the
    /// moment the sub flow fails -- a real action aborts the same way.
    async fn invoke(
        &self,
        execution_identifier: &str,
        input: Value,
    ) -> Result<Value, RuntimeError> {
        let report = self
            .engine
            .execute_sub_flow(execution_identifier, vec![input], Some(self), false)
            .await
            .ok_or_else(|| {
                RuntimeError::new(
                    "T-TEST-000005",
                    "SubFlowNotFound",
                    format!(
                        "Sub flow {} was not minted (or already resolved)",
                        execution_identifier
                    ),
                )
            })?;

        match report.signal {
            Signal::Success(value) | Signal::Return(value) => Ok(value),
            Signal::Failure(err) => Err(err),
            Signal::Stop => Ok(null()),
        }
    }
}

/// Builds the `SubFlowFunctionResult` `remote::sub_flow_function` returns:
/// a reference to `function_identifier`, run wherever `definition_source`
/// says (`null` for Taurus' own functions).
fn sub_flow_function(
    parameters: &[ActionNodeValue],
) -> Result<NodeValueExecutionResult, RuntimeError> {
    let function_identifier = match literal(parameters, 0)?.kind {
        Some(Kind::StringValue(identifier)) => identifier,
        _ => return Err(invalid_parameter(0, "a function identifier string")),
    };
    let definition_source = match literal(parameters, 1)?.kind {
        Some(Kind::StringValue(source)) => Some(source),
        Some(Kind::NullValue(_)) | None => None,
        _ => return Err(invalid_parameter(1, "a definition source string or null")),
    };

    Ok(NodeValueExecutionResult {
        value: Some(node_value_execution_result::Value::SubFlowFunctionResult(
            SubFlowFunctionResult {
                function: Some(SubFlowFunction {
                    function_identifier: function_identifier.clone(),
                    definition_source,
                }),
                input_schema: None,
                output_schema: None,
                name: function_identifier,
                description: "Returned by remote::sub_flow_function".to_string(),
            },
        )),
    })
}

fn raw(value: Value) -> NodeValueExecutionResult {
    raw_value_result(value)
}

fn literal(parameters: &[ActionNodeValue], index: usize) -> Result<Value, RuntimeError> {
    match parameters
        .get(index)
        .and_then(|parameter| parameter.value.as_ref())
    {
        // Fixtures never exercise `${signature}` templating, so the
        // literal's `value` is always already the concrete parameter.
        Some(action_node_value::Value::LiteralValue(literal)) => {
            Ok(literal.value.clone().unwrap_or_else(null))
        }
        _ => Err(invalid_parameter(index, "a literal value")),
    }
}

fn sub_flow(parameters: &[ActionNodeValue], index: usize) -> Result<String, RuntimeError> {
    match parameters
        .get(index)
        .and_then(|parameter| parameter.value.as_ref())
    {
        Some(action_node_value::Value::SubFlow(ActionNodeSubFlowValue {
            execution_identifier,
            ..
        })) => Ok(execution_identifier.clone()),
        _ => Err(invalid_parameter(index, "a sub flow")),
    }
}

fn list(value: &Value) -> Result<Vec<Value>, RuntimeError> {
    match &value.kind {
        Some(Kind::ListValue(list)) => Ok(list.values.clone()),
        _ => Err(invalid_parameter(0, "a list")),
    }
}

fn double(value: &Value) -> Result<Value, RuntimeError> {
    match &value.kind {
        Some(Kind::NumberValue(NumberValue {
            number: Some(number_value::Number::Integer(integer)),
        })) => Ok(value_from_i64(integer * 2)),
        Some(Kind::NumberValue(number)) => number_to_f64(number)
            .map(|number| value_from_f64(number * 2.0))
            .ok_or_else(|| invalid_parameter(0, "a number")),
        _ => Err(invalid_parameter(0, "a number")),
    }
}

fn invalid_parameter(index: usize, expected: &str) -> RuntimeError {
    RuntimeError::new(
        "T-TEST-000003",
        "SimulatedActionParameterInvalid",
        format!("Parameter {} must be {}", index, expected),
    )
}

fn string(value: &str) -> Value {
    Value {
        kind: Some(Kind::StringValue(value.to_string())),
    }
}

fn null() -> Value {
    Value {
        kind: Some(Kind::NullValue(0)),
    }
}
