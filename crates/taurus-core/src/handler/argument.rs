//! Handler argument representation and typed extraction contracts.

use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use std::convert::Infallible;
use tucana::shared::value::Kind;
use tucana::shared::{ListValue, NumberValue, Struct, Value};

use crate::value::{number_to_f64, number_to_i64_lossy};
#[derive(Clone, Debug)]
pub enum Argument {
    /// Eager value that can be consumed immediately by a handler.
    Eval(Value),
    /// Deferred node execution handle, evaluated by calling `run(node_id)`.
    Thunk(i64),
}

#[derive(Clone, Copy, Debug)]
pub enum ParameterNode {
    /// Argument must be resolved before the handler is called.
    Eager,
    /// Argument is passed as a thunk and may be executed by the handler.
    Lazy,
}

/// Conversion interface used by `args!` to parse typed handler inputs.
pub trait TryFromArgument: Sized {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal>;
}

fn type_err(msg: &str, a: &Argument) -> Signal {
    Signal::Failure(RuntimeError::new(
        "T-RT-000000",
        "InvalidArgumentRuntimeError",
        format!("{} but it was the arugment: {:?}", msg, a),
    ))
}

impl TryFromArgument for Value {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(v) => Ok(v.clone()),
            _ => Err(type_err("Expected evaluated value but got lazy thunk", a)),
        }
    }
}

impl TryFromArgument for NumberValue {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => Ok(*n),
            _ => Err(type_err("Expected number", a)),
        }
    }
}

impl TryFromArgument for i64 {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_i64_lossy(n).ok_or_else(|| type_err("Expected number", a)),
            _ => Err(type_err("Expected number", a)),
        }
    }
}

impl TryFromArgument for f64 {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::NumberValue(n)),
            }) => number_to_f64(n).ok_or_else(|| type_err("Expected number", a)),
            _ => Err(type_err("Expected number", a)),
        }
    }
}

impl TryFromArgument for bool {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::BoolValue(b)),
            }) => Ok(*b),
            _ => Err(type_err("Expected boolean", a)),
        }
    }
}

impl TryFromArgument for String {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::StringValue(s)),
            }) => Ok(s.clone()),
            _ => Err(type_err("Expected string", a)),
        }
    }
}

impl TryFromArgument for Struct {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::StructValue(s)),
            }) => Ok(s.clone()),
            _ => Err(type_err("Expected struct", a)),
        }
    }
}

impl TryFromArgument for ListValue {
    fn try_from_argument(a: &Argument) -> Result<Self, Signal> {
        match a {
            Argument::Eval(Value {
                kind: Some(Kind::ListValue(list)),
            }) => Ok(list.clone()),
            _ => Err(Signal::Failure(RuntimeError::new(
                "T-RT-000000",
                "InvalidArgumentRuntimeError",
                format!("Expected array (ListValue) but it was: {:?}", a),
            ))),
        }
    }
}

impl From<Infallible> for RuntimeError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}
