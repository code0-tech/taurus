//! General (application-level) errors that can occur outside pure node execution.
//!
//! Every variant can be converted into a [`RuntimeError`] to unify reporting.

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use tucana::shared::Value;
use tucana::shared::value::Kind::StringValue;

use crate::types::errors::runtime_error::RuntimeError;

/// Application-layer failures that should still be reportable as runtime failures.
#[derive(Debug, Clone)]
pub enum Error {
    /// Invalid or missing runtime configuration.
    Configuration {
        message: String,
        details: HashMap<String, Value>,
    },
    /// Invalid application state transition.
    State {
        message: String,
        details: HashMap<String, Value>,
    },
    /// Failed communication with dependency or transport layer.
    Transport {
        dependency: String,
        message: String,
        details: HashMap<String, Value>,
    },
    /// Failed serialization/deserialization.
    Serialization {
        format: String,
        message: String,
        details: HashMap<String, Value>,
    },
    /// Catch-all internal application error.
    Internal {
        message: String,
        details: HashMap<String, Value>,
    },
}

impl Error {
    /// Build a configuration error with optional structured details.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Build an invalid state error with optional structured details.
    pub fn state(message: impl Into<String>) -> Self {
        Self::State {
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Build a transport/dependency error with optional structured details.
    pub fn transport(dependency: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Transport {
            dependency: dependency.into(),
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Build a serialization error with optional structured details.
    pub fn serialization(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Serialization {
            format: format.into(),
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Build an internal application error with optional structured details.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            details: HashMap::new(),
        }
    }

    /// Attach a detail entry to this error.
    pub fn with_detail(mut self, key: impl Into<String>, value: Value) -> Self {
        match &mut self {
            Error::Configuration { details, .. }
            | Error::State { details, .. }
            | Error::Transport { details, .. }
            | Error::Serialization { details, .. }
            | Error::Internal { details, .. } => {
                details.insert(key.into(), value);
            }
        }
        self
    }
}

impl From<Error> for RuntimeError {
    fn from(value: Error) -> Self {
        match value {
            Error::Configuration { message, details } => {
                let mut err = RuntimeError::with_code(
                    "T-CORE-000301".to_string(),
                    "Configuration".to_string(),
                    message,
                );
                err.details.extend(details);
                err
            }
            Error::State { message, details } => {
                let mut err = RuntimeError::with_code(
                    "T-CORE-000302".to_string(),
                    "State".to_string(),
                    message,
                );
                err.details.extend(details);
                err
            }
            Error::Transport {
                dependency,
                message,
                details,
            } => {
                let mut err = RuntimeError::with_code(
                    "T-CORE-000303".to_string(),
                    "Transport".to_string(),
                    message,
                )
                .with_detail(
                    "dependency".to_string(),
                    Value {
                        kind: Some(StringValue(dependency)),
                    },
                );
                err.details.extend(details);
                err
            }
            Error::Serialization {
                format,
                message,
                details,
            } => {
                let mut err = RuntimeError::with_code(
                    "T-CORE-000304".to_string(),
                    "Serialization".to_string(),
                    message,
                )
                .with_detail(
                    "format".to_string(),
                    Value {
                        kind: Some(StringValue(format)),
                    },
                );
                err.details.extend(details);
                err
            }
            Error::Internal { message, details } => {
                let mut err = RuntimeError::with_code(
                    "T-CORE-000399".to_string(),
                    "Internal".to_string(),
                    message,
                );
                err.details.extend(details);
                err
            }
        }
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Configuration { message, .. } => write!(f, "Configuration error: {message}"),
            Error::State { message, .. } => write!(f, "State error: {message}"),
            Error::Transport {
                dependency,
                message,
                ..
            } => write!(f, "Transport error ({dependency}): {message}"),
            Error::Serialization {
                format, message, ..
            } => write!(f, "Serialization error ({format}): {message}"),
            Error::Internal { message, .. } => write!(f, "Internal error: {message}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucana::shared::{Struct, value::Kind::StructValue};

    #[test]
    fn app_error_converts_to_runtime_error_with_expected_code_and_category() {
        let app_err = Error::transport("nats", "connection lost").with_detail(
            "subject",
            Value {
                kind: Some(StringValue("execution.*".to_string())),
            },
        );

        let runtime_error: RuntimeError = app_err.into();

        assert_eq!(runtime_error.code, "T-CORE-000303");
        assert_eq!(runtime_error.category, "Transport");
        assert_eq!(runtime_error.message, "connection lost");
        assert!(runtime_error.details.contains_key("dependency"));
        assert!(runtime_error.details.contains_key("subject"));
    }

    #[test]
    fn runtime_error_value_contains_required_struct_fields() {
        let runtime_error: RuntimeError = Error::internal("boom").into();
        let value = runtime_error.as_value();

        let Some(StructValue(Struct { fields })) = value.kind else {
            panic!("expected struct value");
        };

        assert!(fields.contains_key("code"));
        assert!(fields.contains_key("category"));
        assert!(fields.contains_key("message"));
        assert!(fields.contains_key("timestamp"));
        assert!(fields.contains_key("version"));
        assert!(fields.contains_key("dependencies"));
        assert!(fields.contains_key("details"));
    }
}
