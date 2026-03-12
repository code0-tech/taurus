use std::collections::HashMap;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};
use tucana::shared::value::Kind::{StringValue, StructValue};
use tucana::shared::{Struct, Value};

#[derive(Debug, Default, Clone)]
pub struct RuntimeError {
    name: String,
    message: String,
    suggestion: Option<String>,
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "&self.function_name.as_str()")
    }
}

impl RuntimeError {
    pub fn new(name: String, message: String, suggestion: Option<String>) -> Self {
        Self {
            name,
            message,
            suggestion,
        }
    }

    pub fn simple_str(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            message: message.to_string(),
            suggestion: None,
        }
    }

    pub fn simple(name: &str, message: String) -> Self {
        Self {
            name: name.to_string(),
            message,
            suggestion: None,
        }
    }

    pub fn as_value(&self) -> Value {
        let suggestion = match self.suggestion {
            Some(ref s) => Value {
                kind: Some(StringValue(s.clone())),
            },
            None => Value { kind: None },
        };

        Value {
            kind: Some(StructValue(Struct {
                fields: HashMap::from([
                    (
                        String::from("name"),
                        Value {
                            kind: Some(StringValue(self.name.clone())),
                        },
                    ),
                    (
                        String::from("message"),
                        Value {
                            kind: Some(StringValue(self.message.clone())),
                        },
                    ),
                    (String::from("suggestion"), suggestion),
                ]),
            })),
        }
    }
}
