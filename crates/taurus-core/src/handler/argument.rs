//! Handler argument representation and typed extraction contracts.

use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use std::convert::Infallible;
use tucana::shared::value::Kind;
use tucana::shared::{ListValue, NumberValue, Struct, Value};

use crate::value::{number_to_f64, number_to_i64_lossy};
use std::fmt;
use tucana::shared::SubFlowSetting;

#[derive(Clone)]
pub struct FunctionThunk {
    pub identifier: String,
    pub result_id: Option<i64>,
    pub parameter_index: i64,
    pub settings: Vec<SubFlowSetting>,
}

impl fmt::Debug for FunctionThunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionThunk")
            .field("identifier", &self.identifier)
            .field("result_id", &self.result_id)
            .field("parameter_index", &self.parameter_index)
            .field("settings_len", &self.settings.len())
            .finish()
    }
}

#[derive(Clone)]
pub enum Thunk {
    Node(i64),
    Function(FunctionThunk),
}

impl fmt::Debug for Thunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Thunk::Node(node_id) => write!(f, "{}", node_id),
            Thunk::Function(function) => function.fmt(f),
        }
    }
}

impl Thunk {
    pub fn trace_target(&self) -> String {
        match self {
            Thunk::Node(node_id) => format!("node={}", node_id),
            Thunk::Function(function) => format!("function={}", function.identifier),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Argument {
    /// Eager value that can be consumed immediately by a handler.
    Eval(Value),
    /// Deferred execution handle, evaluated by calling `run(thunk)`.
    Thunk(Thunk),
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
        "T-CORE-000202",
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
                "T-CORE-000202",
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
