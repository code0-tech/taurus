//! Fills missing/`None` proto value fields with explicit null defaults.
//!
//! Proto3 leaves optional scalar fields unset rather than null; consumers
//! that expect every field to be present (e.g. JSON reporting, debug
//! printing) need `None` normalized to an explicit `NullValue` first.

use tucana::shared::{NodeExecutionResult, Value, node_execution_result, value::Kind};

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

pub fn normalize_node_execution_result(result: &mut NodeExecutionResult) {
    for parameter_result in &mut result.parameter_results {
        match &mut parameter_result.value {
            Some(value) => normalize_value(value),
            None => {
                parameter_result.value = Some(null_value());
            }
        }
    }

    match &mut result.result {
        Some(node_execution_result::Result::Success(value)) => normalize_value(value),
        Some(node_execution_result::Result::Error(error)) => {
            if let Some(details) = &mut error.details {
                for value in details.fields.values_mut() {
                    normalize_value(value);
                }
            }
        }
        None => {
            result.result = Some(node_execution_result::Result::Success(null_value()));
        }
    }
}
