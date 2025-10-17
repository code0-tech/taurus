use tucana::shared::Value;
use crate::error::RuntimeError;

#[derive(Debug)]
pub enum Signal {
    Success(Value),
    Failure(RuntimeError),
    Return(Value),
    Stop
}

impl PartialEq for Signal {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Signal::Success(_), Signal::Success(_))
                | (Signal::Failure(_), Signal::Failure(_))
                | (Signal::Return(_), Signal::Return(_))
                | (Signal::Stop, Signal::Stop)
        )
    }
}