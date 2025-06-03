use std::{
    error::Error,
    fmt::{Display, Formatter},
};

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
            message: message,
            suggestion: None,
        }
    }
}
