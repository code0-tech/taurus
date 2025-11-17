use crate::context::context::Context;
use crate::context::argument::Argument;
use crate::context::argument::ParameterNode::{Eager, Lazy};
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::error::RuntimeError;
use tucana::shared::Value;
use tucana::shared::value::Kind;

pub fn collect_control_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        ("std::control::stop", HandlerFn::eager(stop, 0)),
        ("std::control::return", HandlerFn::eager(r#return, 1)),
        (
            "std::control::if",
            HandlerFn::into_function_entry(r#if, vec![Eager, Lazy]),
        ),
        (
            "std::control::if_else",
            HandlerFn::into_function_entry(if_else, vec![Eager, Lazy, Lazy]),
        ),
    ]
}

fn stop(_args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    Signal::Stop
}

fn r#return(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    args!(args => value: Value);
    Signal::Return(value)
}

fn r#if(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(text)),
        }),
        Argument::Thunk(if_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a string value but received {:?}", args),
        ));
    };

    let bool: bool = match text.to_lowercase().parse() {
        Ok(value) => value,
        Err(_) => {
            return Signal::Failure(RuntimeError::simple(
                "InvalidArgumentRuntimeError",
                format!("Failed to parse boolean from string: {:?}", text),
            ));
        }
    };

    if bool {
        _run(*if_pointer)
    } else {
        Signal::Return(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }
}

fn if_else(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64) -> Signal) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(text)),
        }),
        Argument::Thunk(if_pointer),
        Argument::Thunk(else_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a string value but received {:?}", args),
        ));
    };

    let bool: bool = match text.to_lowercase().parse() {
        Ok(value) => value,
        Err(_) => {
            return Signal::Failure(RuntimeError::simple(
                "InvalidArgumentRuntimeError",
                format!("Failed to parse boolean from string: {:?}", text),
            ));
        }
    };

    if bool {
        _run(*if_pointer)
    } else {
        _run(*else_pointer)
    }
}
