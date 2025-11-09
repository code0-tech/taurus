use std::convert::Infallible;
use tucana::shared::{ListValue, Struct, Value};
use tucana::shared::value::Kind;
use crate::context::signal::Signal;
use crate::error::RuntimeError;

#[derive(Clone, Debug)]
pub enum Argument {
    // Eval => Evaluated Value
    // - can be consumed directly by a function
    Eval(tucana::shared::Value),
    // Thunk of NodeFunction identifier
    // - used for lazy execution of nodes
    Thunk(i64)
}

#[derive(Clone, Copy, Debug)]
pub enum ParameterNode {
    Eager,
    Lazy
}

pub trait TryFromArgument: Sized {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal>;
}

fn type_err(msg: &str) -> Signal {
    Signal::Failure(RuntimeError::simple("InvalidArgumentRuntimeError", msg.to_string()))
}

impl TryFromArgument for Value {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(v) => Ok(v.clone()),
            _ => Err(type_err("Expected evaluated value but got lazy thunk")),
        }
    }
}

impl TryFromArgument for f64 {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value { kind: Some(Kind::NumberValue(n)) }) => Ok(*n),
            _ => Err(type_err("Expected number")),
        }
    }
}

impl TryFromArgument for bool {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value { kind: Some(Kind::BoolValue(b)) }) => Ok(*b),
            _ => Err(type_err("Expected boolean")),
        }
    }
}

impl TryFromArgument for String {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value { kind: Some(Kind::StringValue(s)) }) => Ok(s.clone()),
            _ => Err(type_err("Expected string")),
        }
    }
}

impl TryFromArgument for Struct {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value { kind: Some(Kind::StructValue(s)) }) => Ok(s.clone()),
            _ => Err(type_err("Expected struct")),
        }
    }
}

impl TryFromArgument for ListValue {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value { kind: Some(Kind::ListValue(list)) }) => Ok(list.clone()),
            _ => Err(Signal::Failure(RuntimeError::simple_str(
                "InvalidArgumentRuntimeError",
                "Expected array (ListValue)",
            ))),
        }
    }
}

impl From<Infallible> for RuntimeError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}