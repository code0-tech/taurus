//! Control-flow handlers (`if`, `if_else`, `return`, `stop`).
//!
//! `if`/`if_else` execute branch nodes via runtime callbacks and forward their resulting signals.
//! This is required for block-style return semantics where `return` exits only the current call frame.

use crate::handler::argument::Argument;
use crate::handler::argument::ParameterNode::{Eager, Lazy};
use crate::handler::macros::args;
use crate::handler::registry::FunctionRegistration;
use crate::runtime::execution::value_store::ValueStore;
use crate::types::errors::runtime_error::RuntimeError;
use crate::types::signal::Signal;
use tucana::shared::Value;
use tucana::shared::value::Kind;

pub(crate) const FUNCTIONS: &[FunctionRegistration] = &[
    FunctionRegistration::eager("std::control::stop", stop, 0),
    FunctionRegistration::eager("std::control::return", r#return, 1),
    FunctionRegistration::modes("std::control::if", r#if, &[Eager, Lazy]),
    FunctionRegistration::modes("std::control::if_else", if_else, &[Eager, Lazy, Lazy]),
];

fn stop(
    _args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    Signal::Stop
}

fn r#return(
    args: &[Argument],
    _ctx: &mut ValueStore,
    _run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    args!(args => value: Value);
    // The executor decides how far this return unwinds (one frame).
    Signal::Return(value)
}

fn r#if(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        // Branch execution is delegated to the executor through `run`.
        ctx.push_runtime_trace_label("branch=if".to_string());
        run(*if_pointer, ctx)
    } else {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }
}

fn if_else(
    args: &[Argument],
    ctx: &mut ValueStore,
    run: &mut dyn FnMut(i64, &mut ValueStore) -> Signal,
) -> Signal {
    let [
        Argument::Eval(Value {
            kind: Some(Kind::BoolValue(bool)),
        }),
        Argument::Thunk(if_pointer),
        Argument::Thunk(else_pointer),
    ] = args
    else {
        return Signal::Failure(RuntimeError::new(
            "T-STD-00001",
            "InvalidArgumentRuntimeError",
            format!("Expected a bool value but received {:?}", args),
        ));
    };

    if *bool {
        ctx.push_runtime_trace_label("branch=if".to_string());
        run(*if_pointer, ctx)
    } else {
        ctx.push_runtime_trace_label("branch=else".to_string());
        run(*else_pointer, ctx)
    }
}
