//! Registry metadata types for runtime handler discovery.

use std::collections::HashMap;

use crate::types::execution::signature::HandlerSignature;

/// Static registry entry metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerRegistration {
    pub handler_id: String,
    pub signature: HandlerSignature,
    pub description: Option<String>,
}

/// Read-only handler metadata registry.
#[derive(Debug, Clone, Default)]
pub struct HandlerRegistry {
    pub handlers: HashMap<String, HandlerRegistration>,
}
