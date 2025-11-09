pub mod argument;
pub mod context;
pub mod executor;
pub mod macros;
pub mod registry;
pub mod signal;

use crate::error::RuntimeError;
use std::{
    collections::{HashMap, VecDeque},
    ops::Index,
};
use tucana::shared::{ReferenceValue, Value};

type NodeResult = Result<Value, RuntimeError>;

pub enum ContextResult {
    // Will return the value / error if present of an executed node
    NodeExecutionResult(NodeResult),

    // Will return the parameter of the node (indexed by the context)
    ParameterResult(Value),
}

#[derive(Clone)]
pub struct ContextEntry {
    result: Result<Value, RuntimeError>,
    parameter: Vec<Value>,
}

impl ContextEntry {
    pub fn new(result: NodeResult, parameter: Vec<Value>) -> Self {
        ContextEntry { result, parameter }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ContextReference {
    // Level referencing the context depth (if a node will execute nodes in itself, e.g. foreach, map...)
    pub primary_level: i32,

    // Level of depth in the current context level (node after node starting_node -> next_node --> next_node --> ending_node)
    pub secondary_level: i32,

    // Index of parameters as input parameter of a node
    pub tertiary_level: Option<i32>,
}

pub struct Context {
    current_context_level: ContextReference,
    /// A stack of environments: layer 0 is the outermost.
    layers: HashMap<ContextReference, ContextEntry>,
    /// Context Snapshot of Past Context
    context_history: VecDeque<(i32, i32)>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new, empty context.
    pub fn new() -> Self {
        Context {
            current_context_level: ContextReference {
                primary_level: 0,
                secondary_level: 0,
                tertiary_level: None,
            },
            layers: HashMap::new(),
            context_history: VecDeque::new(),
        }
    }

    pub fn write_to_context(&mut self, reference: ContextReference, entry: ContextEntry) {
        self.layers.insert(reference, entry);
    }

    pub fn write_to_current_context(&mut self, entry: ContextEntry) {
        self.write_to_context(self.current_context_level.clone(), entry);
    }

    pub fn set_current_context(
        &mut self,
        primary_level: i32,
        seconday_level: i32,
        tertiary_level: Option<i32>,
    ) {
        self.current_context_level.primary_level = primary_level;
        self.current_context_level.secondary_level = seconday_level;
        self.current_context_level.tertiary_level = tertiary_level;
    }

    /// Will indent the context and save the past context
    pub fn next_context(&mut self) {
        let context_snapshot = (
            self.current_context_level.primary_level,
            self.current_context_level.secondary_level,
        );

        self.context_history.push_back(context_snapshot);

        self.current_context_level.primary_level += 1;
        self.current_context_level.secondary_level = 0;
    }

    /// Will return to the parent context and increment the seconday level
    pub fn leave_context(&mut self) {
        let last_snapshot = match self.context_history.pop_back() {
            Some(pair) => pair,
            None => return,
        };

        self.current_context_level.primary_level = last_snapshot.0;
        self.current_context_level.secondary_level = last_snapshot.1 + 1;
    }

    pub fn next_node(&mut self) {
        self.current_context_level.secondary_level += 1;
    }

    // Looks up the current Context
    pub fn get_current_context(&self) -> Option<ContextResult> {
        for (context, value) in self.layers.iter() {
            if context.primary_level != self.current_context_level.primary_level {
                continue;
            }

            if context.secondary_level != self.current_context_level.secondary_level {
                continue;
            }

            if let Some(index) = self.current_context_level.tertiary_level {
                let params = &value.parameter;

                let real_index = index as usize;
                let value = params.index(real_index);
                return Some(ContextResult::ParameterResult(value.clone()));
            }

            return Some(ContextResult::NodeExecutionResult(value.result.clone()));
        }
        None
    }

    /// Looks up the context of a reference
    pub fn get(&self, reference: &ReferenceValue) -> Option<ContextResult> {
        unimplemented!("Implement latest reference pattern from Tucana 0.0.39")
    }

    pub fn is_end(&self) -> bool {
        self.current_context_level.primary_level == 0
    }
}
