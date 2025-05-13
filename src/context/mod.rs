/// A simple context that holds variable bindings in nested layers.
use std::collections::HashMap;
use tucana::shared::{ReferenceValue, Value};

use crate::error::RuntimeError;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ContextReference {
    pub primary_level: i32,
    pub secondary_level: i32,
    pub tertiary_level: Option<i32>,
}

#[derive(Debug)]
pub struct Context {
    current_context_level: ContextReference,
    /// A stack of environments: layer 0 is the outermost.
    layers: HashMap<ContextReference, Result<Value, RuntimeError>>,
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
        }
    }

    pub fn write_to_context(
        &mut self,
        reference: ContextReference,
        result: Result<Value, RuntimeError>,
    ) {
        self.layers.insert(reference, result);
    }

    pub fn write_to_current_context(&mut self, result: Result<Value, RuntimeError>) {
        self.write_to_context(self.current_context_level.clone(), result);
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

    pub fn increment_first_level(&mut self) {
        self.current_context_level.primary_level += 1;
        self.current_context_level.secondary_level = 0;
        self.current_context_level.tertiary_level = None
    }

    pub fn increment_second_level(&mut self) {
        self.current_context_level.secondary_level += 1;
        self.current_context_level.tertiary_level = None
    }

    pub fn increment_third_level(&mut self) {
        match self.current_context_level.tertiary_level {
            Some(v) => {
                self.current_context_level.tertiary_level = Some(v + 1);
            }
            None => self.current_context_level.tertiary_level = Some(0),
        };
    }

    // Looks up the current Context
    pub fn get_current_context(&self) -> Option<&Result<Value, RuntimeError>> {
        for (context, value) in self.layers.iter() {
            if context.primary_level != self.current_context_level.primary_level {
                continue;
            }

            if context.secondary_level != self.current_context_level.secondary_level {
                continue;
            }

            if context.tertiary_level != self.current_context_level.tertiary_level {
                continue;
            }

            return Some(value);
        }
        None
    }

    /// Look up a name, searching from innermost outward.
    pub fn get(&self, reference: &ReferenceValue) -> Option<&Result<Value, RuntimeError>> {
        for (context, value) in self.layers.iter() {
            if context.primary_level != reference.primary_level {
                continue;
            }

            if context.secondary_level != reference.secondary_level {
                continue;
            }

            if context.tertiary_level != reference.tertiary_level {
                continue;
            }

            return Some(value);
        }
        None
    }
}
