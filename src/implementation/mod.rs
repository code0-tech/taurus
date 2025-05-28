use tucana::shared::{Value, value::Kind};

use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::number::add", add),
        ("std::number::multiply", multiply),
        ("std::number::substract", substract),
        ("std::number::devide", devide),
        ("std::number::modulo", modulo),
        ("std::number::abs", abs),
        ("std::number::is_positive", is_positive),
        ("std::number::is_greater", is_greater),
        ("std::number::is_less", is_less),
        ("std::number::is_zero", is_zero),
        ("std::number::from_text", from_text),
        ("std::number::as_text", as_text),
    ]
}

fn add(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs + rhs)),
    })
}

fn multiply(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs * rhs)),
    })
}

fn substract(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs - rhs)),
    })
}

fn devide(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs / rhs)),
    })
}

fn modulo(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs % rhs)),
    })
}

fn abs(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.abs())),
    })
}

fn is_positive(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(!value.is_sign_negative())),
    })
}

fn is_greater(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs > rhs)),
    })
}

fn is_less(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
            ..
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs < rhs)),
    })
}

fn is_zero(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value == &0.0)),
    })
}

fn from_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::StringValue(string_value)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    let value: f64 = match string_value.parse() {
        Ok(result) => result,
        Err(_) => return Err(RuntimeError::default()),
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value)),
    })
}

fn as_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
            ..
        },
    ] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}
