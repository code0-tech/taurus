//! Mutable value store used by runtime execution to resolve references.

use std::collections::HashMap;

use tucana::shared::{InputType, ReferenceValue, Value, value::Kind};

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

    pub fn get_current_node_id(&mut self) -> i64 {
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
}
