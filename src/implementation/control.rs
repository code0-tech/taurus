use tucana::shared::Value;

use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect_control_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::control::break", r#break),
        ("std::control::return", r#return),
    ]
}

fn r#break(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value { kind }] = values else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one generic value but received {:?}", values),
        ));
    };

    Ok(Value { kind: kind.clone() })
}

fn r#return(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value { kind }] = values else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one generic value but received {:?}", values),
        ));
    };

    Ok(Value { kind: kind.clone() })
}
