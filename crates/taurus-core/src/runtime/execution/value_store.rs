//! Mutable value store used by runtime execution to resolve references.

use std::collections::HashMap;
use std::sync::Arc;

use tucana::shared::node_execution_result::{Id as TucanaNodeResultId, Result as TucanaNodeResult};
use tucana::shared::{
    InputType, NodeExecutionResult, NodeValueExecutionResult, ReferenceValue,
    SubFlowFunctionResult, Value, node_value_execution_result, value::Kind,
};

use crate::normalize::null_value;
use crate::runtime::execution::trace::{StoreInputSlotEntry, StoreResultEntry, StoreSnapshot};
use crate::types::errors::runtime_error::RuntimeError;
use crate::value::{raw_value_of, raw_value_result};

#[derive(Clone)]
pub enum ValueStoreResult {
    Error(RuntimeError),
    Success(Value),
    /// The referenced node returned a sub flow function (only remote
    /// runtimes do). It isn't a `Value`: the consumer receives it as a sub
    /// flow and decides whether and when to run it.
    SubFlowFunction(Arc<SubFlowFunctionResult>),
    NotFound,
}

#[derive(Default)]
pub struct ValueStore {
    // Arc-wrapped so `insert_node_result` records the same result into both
    // maps via a cheap refcount bump instead of a deep clone (each node
    // result carries its own parameter_results and value trees).
    latest_results: HashMap<i64, Arc<NodeExecutionResult>>,
    result_history: Vec<Arc<NodeExecutionResult>>,
    input_types: HashMap<InputType, Value>,
    flow_input: Value,
    current_node_id: i64,
    runtime_trace_labels: Vec<String>,
    // Lets push_runtime_trace_label skip building a label when nothing
    // will read it back (see that method for why this matters).
    trace_enabled: bool,
}

impl ValueStore {
    pub fn new(flow_input: Value, trace_enabled: bool) -> Self {
        Self {
            latest_results: HashMap::new(),
            result_history: Vec::new(),
            input_types: HashMap::new(),
            flow_input,
            current_node_id: 0,
            runtime_trace_labels: Vec::new(),
            trace_enabled,
        }
    }

    pub fn get_current_node_id(&self) -> i64 {
        self.current_node_id
    }

    pub fn set_current_node_id(&mut self, node_id: i64) {
        self.current_node_id = node_id;
    }

    pub fn get(&mut self, reference: &ReferenceValue) -> ValueStoreResult {
        let target = match &reference.target {
            Some(target) => target,
            None => return ValueStoreResult::NotFound,
        };

        let result = match *target {
            tucana::shared::reference_value::Target::FlowInput(_) => self.get_flow_input(),
            tucana::shared::reference_value::Target::NodeId(id) => self.get_result(id),
            tucana::shared::reference_value::Target::InputType(input_type) => {
                self.get_input_type(input_type)
            }
        };

        if reference.paths.is_empty() {
            return result;
        }

        if let ValueStoreResult::SubFlowFunction(_) = result {
            // A function has no fields or items to address.
            return ValueStoreResult::NotFound;
        }

        if let ValueStoreResult::Success(value) = result {
            let mut current = value;
            for path in &reference.paths {
                if let Some(index) = path.array_index {
                    match current.kind {
                        Some(ref kind) => match kind {
                            Kind::ListValue(list) => match list.values.get(index as usize) {
                                Some(item) => current = item.clone(),
                                None => return ValueStoreResult::NotFound,
                            },
                            _ => return ValueStoreResult::NotFound,
                        },
                        None => return ValueStoreResult::NotFound,
                    }
                }

                if let Some(field_name) = &path.path {
                    match current.kind {
                        Some(ref kind) => {
                            if let Kind::StructValue(struct_value) = kind {
                                match struct_value.fields.get(field_name) {
                                    Some(item) => current = item.clone(),
                                    None => return ValueStoreResult::NotFound,
                                }
                            }
                        }
                        None => return ValueStoreResult::NotFound,
                    }
                }
            }

            ValueStoreResult::Success(current)
        } else {
            result
        }
    }

    fn get_result(&mut self, id: i64) -> ValueStoreResult {
        match self.latest_results.get(&id) {
            Some(result) => match &result.result {
                Some(TucanaNodeResult::Success(value)) => match &value.value {
                    Some(node_value_execution_result::Value::RawValue(value)) => {
                        ValueStoreResult::Success(value.clone())
                    }
                    Some(node_value_execution_result::Value::SubFlowFunctionResult(function)) => {
                        ValueStoreResult::SubFlowFunction(Arc::new(function.clone()))
                    }
                    // An unset value reads as null, like any unset proto3 field.
                    None => ValueStoreResult::Success(null_value()),
                },
                Some(TucanaNodeResult::Error(error)) => {
                    ValueStoreResult::Error(RuntimeError::from_tucana_error(error))
                }
                None => ValueStoreResult::Error(RuntimeError::new(
                    "T-CORE-000006",
                    "NodeExecutionResultMissingOutcome",
                    format!(
                        "Node {} execution result is missing success/error outcome",
                        id
                    ),
                )),
            },
            None => ValueStoreResult::NotFound,
        }
    }

    fn get_flow_input(&mut self) -> ValueStoreResult {
        ValueStoreResult::Success(self.flow_input.clone())
    }

    fn get_input_type(&mut self, input_type: InputType) -> ValueStoreResult {
        match self.input_types.get(&input_type) {
            Some(value) => ValueStoreResult::Success(value.clone()),
            None => ValueStoreResult::NotFound,
        }
    }

    /// Every input seeded for `parameter_index` of `node_id`, in positional
    /// order up to the first gap -- the arguments a sub flow function
    /// without declared parameters is called with.
    pub fn positional_inputs(&self, node_id: i64, parameter_index: i64) -> Vec<Value> {
        (0..)
            .map_while(|input_index| {
                self.input_types
                    .get(&InputType {
                        node_id,
                        parameter_index,
                        input_index,
                    })
                    .cloned()
            })
            .collect()
    }

    pub fn clear_input_type(&mut self, input_type: InputType) {
        self.input_types.remove(&input_type);
    }

    pub fn insert_input_type(&mut self, input_type: InputType, value: Value) {
        self.input_types.insert(input_type, value);
    }

    pub fn insert_flow_input(&mut self, value: Value) {
        self.flow_input = value;
    }

    pub fn insert_success_with_timing(
        &mut self,
        id: i64,
        value: Value,
        parameter_results: Vec<NodeValueExecutionResult>,
        started_at: i64,
        finished_at: i64,
    ) {
        self.insert_node_result(
            id,
            NodeExecutionResult {
                started_at,
                finished_at,
                parameter_results,
                id: Some(TucanaNodeResultId::NodeId(id)),
                result: Some(TucanaNodeResult::Success(raw_value_result(value))),
            },
        );
    }

    pub fn insert_error_with_timing(
        &mut self,
        id: i64,
        runtime_error: RuntimeError,
        parameter_results: Vec<NodeValueExecutionResult>,
        started_at: i64,
        finished_at: i64,
    ) {
        self.insert_node_result(
            id,
            NodeExecutionResult {
                started_at,
                finished_at,
                parameter_results,
                id: Some(TucanaNodeResultId::NodeId(id)),
                result: Some(TucanaNodeResult::Error(runtime_error.as_tucana_error())),
            },
        );
    }

    pub fn insert_node_result(&mut self, id: i64, mut result: NodeExecutionResult) {
        result.id = Some(TucanaNodeResultId::NodeId(id));
        let result = Arc::new(result);
        self.latest_results.insert(id, result.clone());
        self.result_history.push(result);
    }

    pub fn insert_function_success_with_timing(
        &mut self,
        id: String,
        value: Value,
        parameter_results: Vec<NodeValueExecutionResult>,
        started_at: i64,
        finished_at: i64,
    ) {
        self.result_history.push(Arc::new(NodeExecutionResult {
            started_at,
            finished_at,
            parameter_results,
            id: Some(TucanaNodeResultId::FunctionIdentifier(id)),
            result: Some(TucanaNodeResult::Success(raw_value_result(value))),
        }));
    }

    pub fn insert_function_error_with_timing(
        &mut self,
        id: String,
        runtime_error: RuntimeError,
        parameter_results: Vec<NodeValueExecutionResult>,
        started_at: i64,
        finished_at: i64,
    ) {
        self.result_history.push(Arc::new(NodeExecutionResult {
            started_at,
            finished_at,
            parameter_results,
            id: Some(TucanaNodeResultId::FunctionIdentifier(id)),
            result: Some(TucanaNodeResult::Error(runtime_error.as_tucana_error())),
        }));
    }

    /// Takes ownership of the accumulated result history, leaving it empty.
    /// Callers use this once, at report time, after which the store is
    /// discarded. Each entry unwraps its `Arc` for free (no clone) once
    /// `latest_results` (the only other owner) is cleared first.
    pub fn node_execution_results(&mut self) -> Vec<NodeExecutionResult> {
        self.latest_results.clear();
        std::mem::take(&mut self.result_history)
            .into_iter()
            .map(|entry| Arc::try_unwrap(entry).unwrap_or_else(|arc| (*arc).clone()))
            .collect()
    }

    /// Builds and records a trace label only when tracing is active; the
    /// label is otherwise discarded by `trace_link_child` unread, so callers
    /// (array/comparator callbacks, in a hot per-iteration loop) pass the
    /// formatting as a closure instead of paying for it unconditionally.
    pub fn push_runtime_trace_label(&mut self, label: impl FnOnce() -> String) {
        if self.trace_enabled {
            self.runtime_trace_labels.push(label());
        }
    }

    pub fn pop_runtime_trace_label(&mut self) -> Option<String> {
        self.runtime_trace_labels.pop()
    }

    pub fn trace_snapshot(&self) -> StoreSnapshot {
        let mut results = Vec::with_capacity(self.latest_results.len());
        for (node_id, result) in &self.latest_results {
            let preview = match &result.result {
                Some(TucanaNodeResult::Success(value)) => match raw_value_of(value) {
                    Some(value) => preview_value(value),
                    None => "sub-flow-function-result".to_string(),
                },
                Some(TucanaNodeResult::Error(err)) => {
                    format!("{}:{} {}", err.code, err.category, err.message)
                }
                None => "empty-result".to_string(),
            };
            results.push(StoreResultEntry {
                node_id: *node_id,
                result: (**result).clone(),
                preview,
            });
        }
        results.sort_by_key(|entry| entry.node_id);

        let mut input_slots = Vec::with_capacity(self.input_types.len());
        for (input, value) in &self.input_types {
            input_slots.push(StoreInputSlotEntry {
                node_id: input.node_id,
                parameter_index: input.parameter_index,
                input_index: input.input_index,
                preview: preview_value(value),
            });
        }
        input_slots.sort_by_key(|entry| (entry.node_id, entry.parameter_index, entry.input_index));

        StoreSnapshot {
            current_node_id: self.current_node_id,
            flow_input_preview: preview_value(&self.flow_input),
            results,
            input_slots,
        }
    }
}

fn preview_value(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(Kind::NumberValue(v)) => crate::value::number_to_string(v),
        Some(Kind::BoolValue(v)) => v.to_string(),
        Some(Kind::StringValue(v)) => format!("{:?}", v),
        Some(Kind::NullValue(_)) | None => "null".to_string(),
        Some(Kind::ListValue(list)) => {
            let mut parts = Vec::with_capacity(list.values.len());
            for item in &list.values {
                parts.push(preview_value(item));
            }
            format!("[{}]", parts.join(", "))
        }
        Some(Kind::StructValue(struct_value)) => {
            let mut keys: Vec<_> = struct_value.fields.keys().collect();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                if let Some(field_value) = struct_value.fields.get(key) {
                    parts.push(format!("{:?}: {}", key, preview_value(field_value)));
                }
            }
            format!("{{{}}}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucana::shared::reference_value::Target;
    use tucana::shared::{ReferencePath, SubFlowFunction};

    fn store_with_returned_function(node_id: i64) -> ValueStore {
        let mut store = ValueStore::new(null_value(), false);
        store.insert_node_result(
            node_id,
            NodeExecutionResult {
                result: Some(TucanaNodeResult::Success(NodeValueExecutionResult {
                    value: Some(node_value_execution_result::Value::SubFlowFunctionResult(
                        SubFlowFunctionResult {
                            function: Some(SubFlowFunction {
                                function_identifier: "remote::handler".to_string(),
                                definition_source: Some("action.example".to_string()),
                            }),
                            ..Default::default()
                        },
                    )),
                })),
                ..Default::default()
            },
        );
        store
    }

    fn node_reference(node_id: i64, paths: Vec<ReferencePath>) -> ReferenceValue {
        ReferenceValue {
            target: Some(Target::NodeId(node_id)),
            paths,
        }
    }

    #[test]
    fn returned_sub_flow_function_resolves_as_sub_flow() {
        let mut store = store_with_returned_function(1);

        match store.get(&node_reference(1, Vec::new())) {
            ValueStoreResult::SubFlowFunction(function) => assert_eq!(
                function
                    .function
                    .as_ref()
                    .map(|function| function.function_identifier.as_str()),
                Some("remote::handler")
            ),
            _ => panic!("expected the returned sub flow function"),
        }
    }

    #[test]
    fn path_into_returned_sub_flow_function_is_not_found() {
        let mut store = store_with_returned_function(1);
        let path = ReferencePath {
            path: Some("field".to_string()),
            ..Default::default()
        };

        assert!(matches!(
            store.get(&node_reference(1, vec![path])),
            ValueStoreResult::NotFound
        ));
    }

    #[test]
    fn positional_inputs_stop_at_the_first_gap() {
        let mut store = ValueStore::new(null_value(), false);
        for input_index in [0, 1, 3] {
            store.insert_input_type(
                InputType {
                    node_id: 5,
                    parameter_index: 1,
                    input_index,
                },
                crate::value::value_from_i64(input_index),
            );
        }

        assert_eq!(
            store.positional_inputs(5, 1),
            vec![
                crate::value::value_from_i64(0),
                crate::value::value_from_i64(1)
            ]
        );
        assert!(store.positional_inputs(5, 2).is_empty());
    }
}
