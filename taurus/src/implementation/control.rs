use crate::context::argument::Argument;
use crate::context::argument::ParameterNode::{Eager, Lazy};
use crate::context::context::Context;
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

fn stop(_args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64, &mut Context) -> Signal) -> Signal {
    Signal::Stop
}

fn r#return(args: &[Argument], _ctx: &mut Context, _run: &mut dyn FnMut(i64, &mut Context) -> Signal) -> Signal {
    args!(args => value: Value);
    Signal::Return(value)
}

fn r#if(args: &[Argument], ctx: &mut Context, run: &mut dyn FnMut(i64, &mut Context) -> Signal) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        run(*if_pointer, ctx)
    } else {
        Signal::Return(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }
}

fn if_else(args: &[Argument], ctx: &mut Context, run: &mut dyn FnMut(i64, &mut Context) -> Signal) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
        Argument::Thunk(else_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        run(*if_pointer, ctx)
    } else {
        run(*else_pointer, ctx)
    }
}
