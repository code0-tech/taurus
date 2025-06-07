use std::f64;

use tucana::shared::{Value, value::Kind};

use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect_number_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::number::add", add),
        ("std::number::multiply", multiply),
        ("std::number::substract", substract),
        ("std::number::divide", divide),
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
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs + rhs)),
    })
}

fn multiply(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs * rhs)),
    })
}

fn substract(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs - rhs)),
    })
}

fn divide(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    if rhs == &0.0 {
        return Err(RuntimeError::simple_str(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs / rhs)),
    })
}

fn modulo(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    if rhs == &0.0 {
        return Err(RuntimeError::simple_str(
            "DivisionByZero",
            "You cannot divide by zero",
        ));
    }

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs % rhs)),
    })
}

fn abs(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.abs())),
    })
}

fn is_positive(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(!value.is_sign_negative())),
    })
}

fn is_greater(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs > rhs)),
    })
}

fn is_less(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs < rhs)),
    })
}

fn is_zero(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(value == &0.0)),
    })
}

fn square(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected a number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.powf(2.0))),
    })
}

fn exponential(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(base)),
        },
        Value {
            kind: Some(Kind::NumberValue(exponent)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
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
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(decimal_places)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).ceil() / factor)),
    })
}

fn round_down(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(decimal_places)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).floor() / factor)),
    })
}

fn round(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(decimal_places)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    let factor = 10_f64.powi(decimal_places.clone() as i32);

    Ok(Value {
        kind: Some(Kind::NumberValue((value * factor).round() / factor)),
    })
}

fn square_root(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sqrt())),
    })
}

fn root(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(root)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.powf(root.clone()))),
    })
}

fn log(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(log)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.log(log.clone()))),
    })
}

fn ln(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.ln())),
    })
}

fn from_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::StringValue(string_value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one string as argument but received {:?}", values),
        ));
    };

    let value: f64 = match string_value.parse() {
        Ok(result) => result,
        Err(_) => {
            return Err(RuntimeError::simple(
                "InvalidArgumentRuntimeError",
                format!("Failed to parse string as number: {}", string_value),
            ));
        }
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value)),
    })
}

fn as_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn min(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs.min(rhs.clone()))),
    })
}

fn max(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(lhs.max(rhs.clone()))),
    })
}

fn negate(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(-value)),
    })
}

fn random(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(min)),
        },
        Value {
            kind: Some(Kind::NumberValue(max)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(rand::random_range(
            min.clone()..max.clone(),
        ))),
    })
}

fn sin(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sin())),
    })
}

fn cos(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.cos())),
    })
}

fn tan(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.tan())),
    })
}

fn arcsin(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.asin())),
    })
}

fn arccos(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.acos())),
    })
}

fn arctan(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.atan())),
    })
}
fn sinh(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.sinh())),
    })
}

fn cosh(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!("Expected one number as argument but received {:?}", values),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.cosh())),
    })
}

fn clamp(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(value)),
        },
        Value {
            kind: Some(Kind::NumberValue(min)),
        },
        Value {
            kind: Some(Kind::NumberValue(max)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected three numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.clamp(min.clone(), max.clone()))),
    })
}

fn is_equal(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [
        Value {
            kind: Some(Kind::NumberValue(lhs)),
        },
        Value {
            kind: Some(Kind::NumberValue(rhs)),
        },
    ] = values
    else {
        return Err(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected two numbers as arguments but received {:?}",
                values
            ),
        ));
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use tucana::shared::{Value, value::Kind};

    // Helper function to create a number value
    fn create_number_value(num: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(num)),
        }
    }

    // Helper function to create a string value
    fn create_string_value(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }

    // Helper function to create a bool value
    fn create_bool_value(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }

    // Helper function to create an invalid value (no kind)
    fn create_invalid_value() -> Value {
        Value { kind: None }
    }

    #[test]
    fn test_add_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(5.0), create_number_value(3.0)];
        let result = add(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 8.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_add_runtime_exception() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters
        let values = vec![create_number_value(5.0)];
        let result = add(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value types
        let values = vec![create_string_value("hello"), create_number_value(3.0)];
        let result = add(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid values
        let values = vec![create_invalid_value(), create_number_value(3.0)];
        let result = add(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiply_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(4.0), create_number_value(2.5)];
        let result = multiply(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 10.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_multiply_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true), create_number_value(3.0)];
        let result = multiply(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_substract_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(10.0), create_number_value(4.0)];
        let result = substract(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 6.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_substract_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(5.0)];
        let result = substract(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_divide_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(15.0), create_number_value(3.0)];
        let result = divide(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 5.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_divide_by_zero_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(10.0), create_number_value(0.0)];
        let result = divide(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_divide_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![
            create_string_value("not_a_number"),
            create_number_value(2.0),
        ];
        let result = divide(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(10.0), create_number_value(3.0)];
        let result = modulo(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 1.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_modulo_by_zero_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(10.0), create_number_value(0.0)];
        let result = modulo(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value(), create_number_value(3.0)];
        let result = modulo(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_abs_success() {
        let mut ctx = Context::new();

        // Test positive number
        let values = vec![create_number_value(5.0)];
        let result = abs(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 5.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test negative number
        let values = vec![create_number_value(-7.5)];
        let result = abs(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 7.5),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_abs_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("not_a_number")];
        let result = abs(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_positive_success() {
        let mut ctx = Context::new();

        // Test positive number
        let values = vec![create_number_value(5.0)];
        let result = is_positive(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test negative number
        let values = vec![create_number_value(-5.0)];
        let result = is_positive(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }

        // Test zero
        let values = vec![create_number_value(0.0)];
        let result = is_positive(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_positive_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true)];
        let result = is_positive(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_greater_success() {
        let mut ctx = Context::new();

        // Test greater
        let values = vec![create_number_value(10.0), create_number_value(5.0)];
        let result = is_greater(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test not greater
        let values = vec![create_number_value(3.0), create_number_value(7.0)];
        let result = is_greater(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_greater_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![
            create_number_value(5.0),
            create_string_value("not_a_number"),
        ];
        let result = is_greater(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_less_success() {
        let mut ctx = Context::new();

        // Test less
        let values = vec![create_number_value(3.0), create_number_value(7.0)];
        let result = is_less(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test not less
        let values = vec![create_number_value(10.0), create_number_value(5.0)];
        let result = is_less(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_less_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value(), create_number_value(5.0)];
        let result = is_less(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_zero_success() {
        let mut ctx = Context::new();

        // Test zero
        let values = vec![create_number_value(0.0)];
        let result = is_zero(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test non-zero
        let values = vec![create_number_value(5.0)];
        let result = is_zero(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_zero_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("zero")];
        let result = is_zero(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_square_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(4.0)];
        let result = square(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 16.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_square_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(false)];
        let result = square(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_exponential_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(2.0), create_number_value(3.0)];
        let result = exponential(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 8.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_exponential_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(2.0)];
        let result = exponential(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_pi_success() {
        let mut ctx = Context::new();
        let values = vec![];
        let result = pi(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => {
                assert!((val - std::f64::consts::PI).abs() < f64::EPSILON)
            }
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_euler_success() {
        let mut ctx = Context::new();
        let values = vec![];
        let result = euler(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => {
                assert!((val - std::f64::consts::E).abs() < f64::EPSILON)
            }
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_infinity_success() {
        let mut ctx = Context::new();
        let values = vec![];
        let result = infinity(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!(val.is_infinite() && val.is_sign_positive()),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_round_up_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.14159), create_number_value(2.0)];
        let result = round_up(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.15),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_round_up_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("3.14"), create_number_value(2.0)];
        let result = round_up(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_down_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.14159), create_number_value(2.0)];
        let result = round_down(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.14),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_round_down_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.14), create_invalid_value()];
        let result = round_down(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.14159), create_number_value(2.0)];
        let result = round(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.14),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_round_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true), create_number_value(2.0)];
        let result = round(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_square_root_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(16.0)];
        let result = square_root(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 4.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_square_root_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("sixteen")];
        let result = square_root(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_root_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(8.0), create_number_value(1.0 / 3.0)];
        let result = root(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 2.0).abs() < 0.001),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_root_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(8.0)];
        let result = root(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(100.0), create_number_value(10.0)];
        let result = log(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 2.0).abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_log_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value(), create_number_value(10.0)];
        let result = log(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_ln_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(std::f64::consts::E)];
        let result = ln(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 1.0).abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_ln_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true)];
        let result = ln(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_text_success() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("42.5")];
        let result = from_text(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 42.5),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_from_text_runtime_exception() {
        let mut ctx = Context::new();

        // Test with invalid string
        let values = vec![create_string_value("not_a_number")];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong type
        let values = vec![create_number_value(42.0)];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_as_text_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(42.5)];
        let result = as_text(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "42.5"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_as_text_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("already_text")];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_min_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.0), create_number_value(7.0)];
        let result = min(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_min_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.0), create_bool_value(false)];
        let result = min(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(3.0), create_number_value(7.0)];
        let result = max(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 7.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_max_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("three"), create_number_value(7.0)];
        let result = max(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_negate_success() {
        let mut ctx = Context::new();

        // Test positive number
        let values = vec![create_number_value(5.0)];
        let result = negate(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, -5.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test negative number
        let values = vec![create_number_value(-3.0)];
        let result = negate(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_negate_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value()];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_random_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(1.0), create_number_value(10.0)];
        let result = random(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => {
                assert!(val >= 1.0 && val < 10.0);
            }
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_random_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(1.0), create_string_value("ten")];
        let result = random(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_sin_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(std::f64::consts::PI / 2.0)];
        let result = sin(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 1.0).abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_sin_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true)];
        let result = sin(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_cos_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(0.0)];
        let result = cos(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 1.0).abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_cos_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("zero")];
        let result = cos(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_tan_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(std::f64::consts::PI / 4.0)];
        let result = tan(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 1.0).abs() < 0.0001),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_tan_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value()];
        let result = tan(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_arcsin_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(1.0)];
        let result = arcsin(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => {
                assert!((val - std::f64::consts::PI / 2.0).abs() < f64::EPSILON)
            }
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_arcsin_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(false)];
        let result = arcsin(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_arccos_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(1.0)];
        let result = arccos(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!(val.abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_arccos_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("one")];
        let result = arccos(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_arctan_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(1.0)];
        let result = arctan(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => {
                assert!((val - std::f64::consts::PI / 4.0).abs() < f64::EPSILON)
            }
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_arctan_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_invalid_value()];
        let result = arctan(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_sinh_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(0.0)];
        let result = sinh(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!(val.abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_sinh_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_bool_value(true)];
        let result = sinh(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_cosh_success() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(0.0)];
        let result = cosh(&values, &mut ctx).unwrap();

        match result.kind {
            Some(Kind::NumberValue(val)) => assert!((val - 1.0).abs() < f64::EPSILON),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_cosh_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_string_value("zero")];
        let result = cosh(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_clamp_success() {
        let mut ctx = Context::new();

        // Test value within range
        let values = vec![
            create_number_value(5.0),
            create_number_value(1.0),
            create_number_value(10.0),
        ];
        let result = clamp(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 5.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test value below range
        let values = vec![
            create_number_value(-5.0),
            create_number_value(1.0),
            create_number_value(10.0),
        ];
        let result = clamp(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 1.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test value above range
        let values = vec![
            create_number_value(15.0),
            create_number_value(1.0),
            create_number_value(10.0),
        ];
        let result = clamp(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 10.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_clamp_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(5.0), create_string_value("one")];
        let result = clamp(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_equal_success() {
        let mut ctx = Context::new();

        // Test equal numbers
        let values = vec![create_number_value(5.0), create_number_value(5.0)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test unequal numbers
        let values = vec![create_number_value(5.0), create_number_value(3.0)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_equal_runtime_exception() {
        let mut ctx = Context::new();
        let values = vec![create_number_value(5.0), create_bool_value(true)];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());
    }
}
