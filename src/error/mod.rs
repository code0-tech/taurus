use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Default, Clone)]
pub struct RuntimeError {}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "&self.function_name.as_str()")
    }
}
