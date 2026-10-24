//! Fills missing/`None` proto value fields with explicit null defaults.
//!
//! Proto3 leaves optional scalar fields unset rather than null; consumers
//! that expect every field to be present (e.g. JSON reporting, debug
//! printing) need `None` normalized to an explicit `NullValue` first.

use tucana::shared::{
    NodeExecutionResult, NodeValueExecutionResult, Value, node_execution_result,
    node_value_execution_result, value::Kind,
};

pub fn null_value() -> Value {
    Value {
        kind: Some(Kind::NullValue(0)),
    }
}

pub fn normalize_value(value: &mut Value) {
    match &mut value.kind {
        Some(Kind::StructValue(struct_value)) => {
            for field in struct_value.fields.values_mut() {
                normalize_value(field);
            }
        }
        Some(Kind::ListValue(list_value)) => {
            for item in &mut list_value.values {
                normalize_value(item);
            }
        }
        Some(Kind::NumberValue(number)) if number.number.is_none() => {
            value.kind = Some(Kind::NullValue(0));
        }
        Some(_) => {}
        None => {
            value.kind = Some(Kind::NullValue(0));
        }
    }
}

/// Normalizes a node value result: a raw value is normalized in place, an
/// unset one becomes an explicit null raw value. Sub-flow function results
/// carry no `Value` to normalize and are left untouched.
pub fn normalize_node_value_result(result: &mut NodeValueExecutionResult) {
    match &mut result.value {
        Some(node_value_execution_result::Value::RawValue(value)) => normalize_value(value),
        Some(node_value_execution_result::Value::SubFlowFunctionResult(_)) => {}
        None => {
            result.value = Some(node_value_execution_result::Value::RawValue(null_value()));
        }
    }
}

pub fn normalize_node_execution_result(result: &mut NodeExecutionResult) {
    for parameter_result in &mut result.parameter_results {
        normalize_node_value_result(parameter_result);
    }

    match &mut result.result {
        Some(node_execution_result::Result::Success(value)) => normalize_node_value_result(value),
        Some(node_execution_result::Result::Error(error)) => {
            if let Some(details) = &mut error.details {
                for value in details.fields.values_mut() {
                    normalize_value(value);
                }
            }
        }
        None => {
            result.result = Some(node_execution_result::Result::Success(
                NodeValueExecutionResult {
                    value: Some(node_value_execution_result::Value::RawValue(null_value())),
                },
            ));
        }
    }
}
