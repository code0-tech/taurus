use tucana::shared::Value;

use crate::context::signal::Signal;
use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect_control_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::control::stop", stop),
        ("std::control::return", r#return),
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
