//! Numeric/value conversion helpers used by runtime handlers.

use tucana::shared::{
    NodeValueExecutionResult, NumberValue, Value, node_value_execution_result, number_value,
    value::Kind,
};

use crate::normalize::null_value;
use crate::types::errors::runtime_error::RuntimeError;

pub fn number_value_from_f64(n: f64) -> NumberValue {
    NumberValue {
        number: Some(number_value::Number::Float(n)),
    }
}

pub fn number_value_from_i64(n: i64) -> NumberValue {
    NumberValue {
        number: Some(number_value::Number::Integer(n)),
    }
}

pub fn value_from_f64(n: f64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(number_value_from_f64(n))),
    }
}

pub fn value_from_i64(n: i64) -> Value {
    Value {
        kind: Some(Kind::NumberValue(number_value_from_i64(n))),
    }
}

pub fn number_to_f64(n: &NumberValue) -> Option<f64> {
    match n.number {
        Some(number_value::Number::Integer(i)) => Some(i as f64),
        Some(number_value::Number::Float(f)) => Some(f),
        None => None,
    }
}

pub fn number_to_i64_lossy(n: &NumberValue) -> Option<i64> {
    match n.number {
        Some(number_value::Number::Integer(i)) => Some(i),
        Some(number_value::Number::Float(f)) => {
            if f.is_finite() {
                Some(f as i64)
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn number_to_string(n: &NumberValue) -> String {
    match n.number {
        Some(number_value::Number::Integer(i)) => i.to_string(),
        Some(number_value::Number::Float(f)) => f.to_string(),
        None => "null".to_string(),
    }
}

/// Wraps a plain value as the `raw_value` variant every node result produced
/// by this engine uses.
pub fn raw_value_result(value: Value) -> NodeValueExecutionResult {
    NodeValueExecutionResult {
        value: Some(node_value_execution_result::Value::RawValue(value)),
    }
}

/// Returns the plain value carried by a node result, if it has one.
pub fn raw_value_of(result: &NodeValueExecutionResult) -> Option<&Value> {
    match result.value.as_ref() {
        Some(node_value_execution_result::Value::RawValue(value)) => Some(value),
        _ => None,
    }
}

/// Extracts the plain value a node produced; an unset value reads as null,
/// like any other unset proto3 field (see `normalize`). A sub-flow function
/// result (only ever reported by a remote runtime) can't be consumed by the
/// engine yet, so it surfaces as a runtime error instead of a silent null.
pub fn node_value_result_to_value(
    result: &NodeValueExecutionResult,
) -> Result<Value, RuntimeError> {
    match result.value.as_ref() {
        Some(node_value_execution_result::Value::RawValue(value)) => Ok(value.clone()),
        Some(node_value_execution_result::Value::SubFlowFunctionResult(_)) => {
            Err(RuntimeError::new(
                "T-CORE-000007",
                "NodeExecutionResultUnsupportedValue",
                "Sub flow function results are not supported as node values yet",
            ))
        }
        None => Ok(null_value()),
    }
}
