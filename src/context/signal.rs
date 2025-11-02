use crate::error::RuntimeError;
use tucana::shared::Value;

#[derive(Debug)]
pub enum Signal {
    // Will be signaled if a function has been executed successfully
    Success(Value),
    // Will be signaled if
    // - a function recieves wrong parameter
    // - a function throws an error
    // - taurus itself throwns an error
    // - will stop the execution of the flow completly
    Failure(RuntimeError),
    // Will be signaled if the `return` function has been executed
    // - will break the current context and return the value to the upper node
    Return(Value),
    // Will be signaled if the `respond` function has been executed
    // - will stop the execution of the flow completly
    // - will return the value to the adapter
    Respond(Value),
    // Will be signaled if the `stop` function has been executed
    // - will stop the execution of the flow completly
    Stop,
}

impl PartialEq for Signal {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Signal::Success(_), Signal::Success(_))
                | (Signal::Failure(_), Signal::Failure(_))
                | (Signal::Return(_), Signal::Return(_))
                | (Signal::Stop, Signal::Stop)
                | (Signal::Respond(_), Signal::Respond(_))
        )
    }
}
