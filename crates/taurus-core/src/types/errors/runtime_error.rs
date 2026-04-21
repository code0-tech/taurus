//! Runtime-facing error object used as execution failure payload.
//!
//! Format goals:
//! - Stable machine fields (`code`, `category`) for filtering and analytics
//! - Human-readable `message`
//! - Timestamp and version for support/debugging
//! - Optional dependency map and structured details

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use tucana::shared::value::Kind::{NumberValue, StringValue, StructValue};
use tucana::shared::{NumberValue as ProtoNumberValue, Struct, Value, number_value};

/// Runtime execution failure representation.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    /// Three-part error code (example: `T-STD-000123`).
    pub code: String,
    /// Logical category of the error (example: `InvalidArgument`).
    pub category: String,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Unix timestamp in milliseconds when this error object was created.
    pub timestamp_unix_ms: u64,
    /// Runtime version identifier.
    pub version: String,
    /// Dependency versions relevant to this runtime.
    pub dependencies: HashMap<String, String>,
    /// Additional structured context.
    pub details: HashMap<String, Value>,
}

impl RuntimeError {
    /// Build a runtime error from explicit code/category/message.
    pub fn new(
        code: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            category: category.into(),
            message: message.into(),
            timestamp_unix_ms: now_unix_ms(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            dependencies: HashMap::new(),
            details: HashMap::new(),
        }
    }

    /// Build a runtime error from explicit owned fields.
    pub fn with_code(code: String, category: String, message: String) -> Self {
        Self::new(code, category, message)
    }

    /// Attach or overwrite a dependency version entry.
    pub fn with_dependency(mut self, name: String, version: String) -> Self {
        self.dependencies.insert(name, version);
        self
    }

    /// Attach or overwrite a structured detail entry.
    pub fn with_detail(mut self, key: String, value: Value) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Convert to proto `Value` for transport back to callers.
    pub fn as_value(&self) -> Value {
        let dependencies = self
            .dependencies
            .iter()
            .map(|(name, version)| {
                (
                    name.clone(),
                    Value {
                        kind: Some(StringValue(version.clone())),
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        Value {
            kind: Some(StructValue(Struct {
                fields: HashMap::from([
                    (
                        String::from("code"),
                        Value {
                            kind: Some(StringValue(self.code.clone())),
                        },
                    ),
                    (
                        String::from("category"),
                        Value {
                            kind: Some(StringValue(self.category.clone())),
                        },
                    ),
                    (
                        String::from("message"),
                        Value {
                            kind: Some(StringValue(self.message.clone())),
                        },
                    ),
                    (
                        String::from("timestamp"),
                        Value {
                            kind: Some(NumberValue(ProtoNumberValue {
                                number: Some(number_value::Number::Integer(
                                    self.timestamp_unix_ms as i64,
                                )),
                            })),
                        },
                    ),
                    (
                        String::from("version"),
                        Value {
                            kind: Some(StringValue(self.version.clone())),
                        },
                    ),
                    (
                        String::from("dependencies"),
                        Value {
                            kind: Some(StructValue(Struct {
                                fields: dependencies,
                            })),
                        },
                    ),
                    (
                        String::from("details"),
                        Value {
                            kind: Some(StructValue(Struct {
                                fields: self.details.clone(),
                            })),
                        },
                    ),
                ]),
            })),
        }
    }
}

impl Default for RuntimeError {
    fn default() -> Self {
        Self::new("T-RT-000000", "RuntimeError", "Unknown runtime error")
    }
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.category, self.message)
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|it| it.as_millis() as u64)
        .unwrap_or(0)
}
