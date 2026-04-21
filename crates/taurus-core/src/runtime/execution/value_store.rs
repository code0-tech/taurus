//! Mutable value store used by runtime execution to resolve references.

use std::collections::HashMap;

use tucana::shared::{InputType, ReferenceValue, Value, value::Kind};

use crate::runtime::execution::trace::{
    StoreInputSlotEntry, StoreResultEntry, StoreResultStatus, StoreSnapshot,
};
use crate::types::errors::runtime_error::RuntimeError;

#[derive(Clone)]
pub enum ValueStoreResult {
    Error(RuntimeError),
    Success(Value),
    NotFound,
}

#[derive(Default)]
pub struct ValueStore {
    results: HashMap<i64, ValueStoreResult>,
    input_types: HashMap<InputType, Value>,
    flow_input: Value,
    current_node_id: i64,
    runtime_trace_labels: Vec<String>,
}

impl ValueStore {
    pub fn new(flow_input: Value) -> Self {
        Self {
            results: HashMap::new(),
            input_types: HashMap::new(),
            flow_input,
            current_node_id: 0,
            runtime_trace_labels: Vec::new(),
        }
    }

    pub fn get_current_node_id(&self) -> i64 {
        self.current_node_id
    }

    pub fn set_current_node_id(&mut self, node_id: i64) {
        self.current_node_id = node_id;
    }

    pub fn get(&mut self, reference: ReferenceValue) -> ValueStoreResult {
        let target = match reference.target {
            Some(target) => target,
            None => return ValueStoreResult::NotFound,
        };

        let result = match target {
            tucana::shared::reference_value::Target::FlowInput(_) => self.get_flow_input(),
            tucana::shared::reference_value::Target::NodeId(id) => self.get_result(id),
            tucana::shared::reference_value::Target::InputType(input_type) => {
                self.get_input_type(input_type)
            }
        };

        if reference.paths.is_empty() {
            return result;
        }

        if let ValueStoreResult::Success(value) = result {
            let mut current = value;
            for path in reference.paths {
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

                if let Some(field_name) = path.path {
                    match current.kind {
                        Some(ref kind) => {
                            if let Kind::StructValue(struct_value) = kind {
                                match struct_value.fields.get(&field_name) {
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
        match self.results.get(&id) {
            Some(result) => result.clone(),
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

    pub fn clear_input_type(&mut self, input_type: InputType) {
        self.input_types.remove(&input_type);
    }

    pub fn insert_input_type(&mut self, input_type: InputType, value: Value) {
        self.input_types.insert(input_type, value);
    }

    pub fn insert_flow_input(&mut self, value: Value) {
        self.flow_input = value;
    }

    pub fn insert_success(&mut self, id: i64, value: Value) {
        self.results.insert(id, ValueStoreResult::Success(value));
    }

    pub fn insert_error(&mut self, id: i64, runtime_error: RuntimeError) {
        self.results
            .insert(id, ValueStoreResult::Error(runtime_error));
    }

    pub fn push_runtime_trace_label(&mut self, label: String) {
        self.runtime_trace_labels.push(label);
    }

    pub fn pop_runtime_trace_label(&mut self) -> Option<String> {
        self.runtime_trace_labels.pop()
    }

    pub fn trace_snapshot(&self) -> StoreSnapshot {
        let mut results = Vec::with_capacity(self.results.len());
        for (node_id, result) in &self.results {
            match result {
                ValueStoreResult::Success(value) => results.push(StoreResultEntry {
                    node_id: *node_id,
                    status: StoreResultStatus::Success,
                    preview: preview_value(value),
                }),
                ValueStoreResult::Error(err) => results.push(StoreResultEntry {
                    node_id: *node_id,
                    status: StoreResultStatus::Error,
                    preview: format!("{}:{} {}", err.code, err.category, err.message),
                }),
                ValueStoreResult::NotFound => {}
            }
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
