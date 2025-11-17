use crate::error::RuntimeError;
use std::collections::HashMap;
use tucana::shared::Value;

#[derive(Clone)]
pub enum ContextResult {
    Error(RuntimeError),
    Success(Value),
    NotFound,
}

#[derive(Default)]
pub struct Context {
    results: HashMap<i64, ContextResult>,
}

impl Context {
    pub fn get(&mut self, id: i64) -> ContextResult {
        match self.results.get(&id) {
            None => ContextResult::NotFound,
            Some(result) => result.clone(),
        }
    }

    pub fn insert_success(&mut self, id: i64, value: Value) {
        self.results.insert(id, ContextResult::Success(value));
    }

    pub fn insert_error(&mut self, id: i64, runtime_error: RuntimeError) {
        self.results.insert(id, ContextResult::Error(runtime_error));
    }
}
