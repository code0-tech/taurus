use crate::context::signal::Signal;
use crate::{context::Context, error::RuntimeError, registry::HandlerFn};
use tucana::shared::Value;
use tucana::shared::value::Kind;

pub fn collect_control_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::control::stop", stop),
        ("std::control::return", r#return),
        ("std::control::if", r#if),
        ("std::control::if_else", if_else),
    ]
}

fn stop(_values: &[Value], _ctx: &mut Context) -> Signal {
    Signal::Stop
}

fn r#return(values: &[Value], _ctx: &mut Context) -> Signal {
    let [Value { kind }] = values else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one generic value but received {:?}", values),
        ));
    };

    Signal::Return(Value { kind: kind.clone() })
}

fn r#if(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StringValue(text)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a string value but received {:?}", values),
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
        unimplemented!()
    } else {
        unimplemented!()
    }
}

fn if_else(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StringValue(text)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a string value but received {:?}", values),
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
        unimplemented!()
    } else {
        unimplemented!()
    }
}