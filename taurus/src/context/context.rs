use crate::error::RuntimeError;
use std::collections::HashMap;
use tucana::shared::{InputType, ReferenceValue, Value, value::Kind};

#[derive(Clone)]
pub enum ContextResult {
    Error(RuntimeError),
    Success(Value),
    NotFound,
}

#[derive(Default)]
pub struct Context {
    results: HashMap<i64, ContextResult>,
    input_types: HashMap<InputType, Value>,
    flow_input: Value,
    current_node_id: i64,
}

impl Context {
    pub fn new(flow_input: Value) -> Self {
        Context {
            results: HashMap::new(),
            input_types: HashMap::new(),
            flow_input,
            current_node_id: 0,
        }
    }

    pub fn get_current_node_id(&mut self) -> i64 {
        self.current_node_id
    }

    pub fn set_current_node_id(&mut self, node_id: i64) {
        self.current_node_id = node_id;
    }

    pub fn get(&mut self, reference: ReferenceValue) -> ContextResult {
        let target = match reference.target {
            Some(t) => t,
            None => return ContextResult::NotFound,
        };

        let res = match target {
            tucana::shared::reference_value::Target::FlowInput(_) => self.get_flow_input(),
            tucana::shared::reference_value::Target::NodeId(i) => self.get_result(i),
            tucana::shared::reference_value::Target::InputType(input_type) => {
                self.get_input_type(input_type)
            }
        };

        if reference.paths.is_empty() {
            return res;
        }

        if let ContextResult::Success(value) = res {
            let mut curr = value;

            for path in reference.paths {
                if let Some(index) = path.array_index {
                    match curr.kind {
                        Some(ref kind) => {
                            if let Kind::ListValue(list) = &kind {
                                match list.values.get(index as usize) {
                                    Some(x) => {
                                        curr = x.clone();
                                    }
                                    None => return ContextResult::NotFound,
                                }
                            }
                        }
                        None => return ContextResult::NotFound,
                    }
                }

                if let Some(path) = path.path {
                    let splits = path.split(".");

                    for part in splits {
                        match curr.kind {
                            Some(ref kind) => {
                                if let Kind::StructValue(struct_value) = &kind {
                                    match struct_value.fields.get(part) {
                                        Some(x) => {
                                            curr = x.clone();
                                        }
                                        None => return ContextResult::NotFound,
                                    }
                                }
                            }
                            None => return ContextResult::NotFound,
                        }
                    }
                }
            }

            ContextResult::Success(curr)
        } else {
            res
        }
    }

    fn get_result(&mut self, id: i64) -> ContextResult {
        match self.results.get(&id) {
            None => ContextResult::NotFound,
            Some(result) => result.clone(),
        }
    }

    fn get_flow_input(&mut self) -> ContextResult {
        ContextResult::Success(self.flow_input.clone())
    }

    fn get_input_type(&mut self, input_type: InputType) -> ContextResult {
        match self.input_types.get(&input_type) {
            Some(v) => ContextResult::Success(v.clone()),
            None => ContextResult::NotFound,
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
        self.results.insert(id, ContextResult::Success(value));
    }

    pub fn insert_error(&mut self, id: i64, runtime_error: RuntimeError) {
        self.results.insert(id, ContextResult::Error(runtime_error));
    }
}
