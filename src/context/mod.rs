/// A simple context that holds variable bindings in nested layers.
/// Each layer is a map from String identifiers to Values.
use std::collections::HashMap;
use tucana::shared::{ReferenceValue, Value};

use crate::error::RuntimeError;

#[derive(Debug)]
pub struct ContextReference {
    pub primary_level: i32,
    pub secondary_level: i32,
    pub tertiary_level: Option<i32>,
}

#[derive(Debug)]
pub struct Context {
    /// A stack of environments: layer 0 is the outermost.
    layers: HashMap<ContextReference, Result<Value, RuntimeError>>,
}

impl Context {
    /// Create a new, empty context.
    pub fn new() -> Self {
        Context {
            layers: HashMap::new(),
        }
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
