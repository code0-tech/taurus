use std::f64;

use tucana::shared::{value::Kind, Value};

use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect_number_functions() -> Vec<(&'static str, HandlerFn)> {
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
        ("std::number::square", square),
        ("std::number::exponential", exponential),
        ("std::number::pi", pi),
        ("std::number::euler", euler),
        ("std::number::infinity", infinity),
        ("std::number::round_up", round_up),
        ("std::number::round_down", round_down),
        ("std::number::round", round),
        ("std::number::square_root", square_root),
        ("std::number::root", root),
        ("std::number::log", log),
        ("std::number::ln", ln),
        ("std::number::from_text", from_text),
        ("std::number::as_text", as_text),
        ("std::number::min", min),
        ("std::number::max", max),
        ("std::number::negate", negate),
        ("std::number::random", random),
        ("std::number::sin", sin),
        ("std::number::cos", cos),
        ("std::number::tan", tan),
        ("std::number::arcsin", arcsin),
        ("std::number::arccos", arccos),
        ("std::number::arctan", arctan),
        ("std::number::sinh", sinh),
        ("std::number::cosh", cosh),
        ("std::number::clamp", clamp),
        ("std::number::is_equal", is_equal),
    ]
}

fn add(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs + rhs)),
    })
}

fn multiply(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs * rhs)),
    })
}

fn substract(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs - rhs)),
    })
}

fn devide(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    if rhs == &0.0 {
        return Err(RuntimeError::simple(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs / rhs)),
    })
}

fn modulo(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    if rhs == &0.0 {
        return Err(RuntimeError::simple(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs % rhs)),
    })
}

fn abs(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.abs())),
    })
}

fn is_positive(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(!value.is_sign_negative())),
    })
}

fn is_greater(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs > rhs)),
    })
}

fn is_less(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs < rhs)),
    })
}

fn is_zero(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value == &0.0)),
    })
}

fn square(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.powf(2.0))),
    })
}

fn exponential(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(base)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(exponent)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(base.powf(exponent.clone()))),
    })
}

fn pi(_values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    Ok(Value {
        kind: Some(Kind::NumberValue(f64::consts::PI)),
    })
}

fn euler(_values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    Ok(Value {
        kind: Some(Kind::NumberValue(f64::consts::E)),
    })
}

fn infinity(_values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    Ok(Value {
        kind: Some(Kind::NumberValue(f64::INFINITY)),
    })
}

fn round_up(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(decimal_places)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).ceil() / factor)),
    })
}

fn round_down(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(decimal_places)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).floor() / factor)),
    })
}

fn round(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(decimal_places)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).round() / factor)),
    })
}

fn square_root(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sqrt())),
    })
}

fn root(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(root)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.powf(root.clone()))),
    })
}

fn log(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(log)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.log(log.clone()))),
    })
}

fn ln(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.ln())),
    })
}

fn from_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(string_value)),
        ..
    }] = values
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
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn min(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs.min(rhs.clone()))),
    })
}

fn max(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs.max(rhs.clone()))),
    })
}

fn negate(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(-value)),
    })
}

fn random(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(min)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(max)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(rand::random_range(
            min.clone()..max.clone(),
        ))),
    })
}

fn sin(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sin())),
    })
}

fn cos(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.cos())),
    })
}

fn tan(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.tan())),
    })
}

fn arcsin(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.asin())),
    })
}

fn arccos(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.acos())),
    })
}

fn arctan(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.atan())),
    })
}
fn sinh(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sinh())),
    })
}

fn cosh(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.cosh())),
    })
}

fn clamp(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(value)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(min)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(max)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.clamp(min.clone(), max.clone()))),
    })
}

fn is_equal(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(lhs)),
        ..
    }, Value {
        kind: Some(Kind::NumberValue(rhs)),
        ..
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}
