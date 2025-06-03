use crate::{context::Context, error::RuntimeError, registry::HandlerFn};
use tucana::shared::{value::Kind, Value};

pub fn collect_boolean_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::boolean::as_number", as_number),
        ("std::boolean::as_text", as_text),
        ("std::boolean::from_number", from_number),
        ("std::boolean::from_text", from_text),
        ("std::boolean::is_equal", is_equal),
        ("std::number::negate", negate),
    ]
}

fn as_number(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::BoolValue(value)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::NumberValue(value.clone() as i64 as f64)),
    })
}

fn as_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::BoolValue(value)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::StringValue(value.to_string())),
    })
}

fn from_number(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::NumberValue(number)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    let is_zero = number == &0.0;

    Ok(Value {
        kind: Some(Kind::BoolValue(!is_zero)),
    })
}

fn from_text(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::StringValue(text)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    let bool: bool = match text.to_lowercase().parse() {
        Ok(value) => value,
        Err(_) => return Err(RuntimeError::default()),
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(bool)),
    })
}

fn is_equal(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::BoolValue(lhs)),
    }, Value {
        kind: Some(Kind::BoolValue(rhs)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(lhs == rhs)),
    })
}

fn negate(values: &[Value], _ctx: &mut Context) -> Result<Value, RuntimeError> {
    let [Value {
        kind: Some(Kind::BoolValue(value)),
    }] = values
    else {
        return Err(RuntimeError::default());
    };

    Ok(Value {
        kind: Some(Kind::BoolValue(!value)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use tucana::shared::{value::Kind, Value};

    // Helper function to create a bool value
    fn create_bool_value(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }

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

    // Helper function to create an invalid value (no kind)
    fn create_invalid_value() -> Value {
        Value { kind: None }
    }

    #[test]
    fn test_as_number_success() {
        let mut ctx = Context::new();

        // Test true -> 1.0
        let values = vec![create_bool_value(true)];
        let result = as_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 1.0),
            _ => panic!("Expected NumberValue"),
        }

        // Test false -> 0.0
        let values = vec![create_bool_value(false)];
        let result = as_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 0.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_as_number_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = as_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![create_bool_value(true), create_bool_value(false)];
        let result = as_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (number)
        let values = vec![create_number_value(5.0)];
        let result = as_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (string)
        let values = vec![create_string_value("hello")];
        let result = as_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = as_number(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_as_text_success() {
        let mut ctx = Context::new();

        // Test true -> "true"
        let values = vec![create_bool_value(true)];
        let result = as_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "true"),
            _ => panic!("Expected StringValue"),
        }

        // Test false -> "false"
        let values = vec![create_bool_value(false)];
        let result = as_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::StringValue(val)) => assert_eq!(val, "false"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_as_text_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![create_bool_value(true), create_bool_value(false)];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (number)
        let values = vec![create_number_value(5.0)];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (string)
        let values = vec![create_string_value("hello")];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = as_text(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_number_success() {
        let mut ctx = Context::new();

        // Test 0.0 -> false
        let values = vec![create_number_value(0.0)];
        let result = from_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }

        // Test positive number -> true
        let values = vec![create_number_value(5.0)];
        let result = from_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test negative number -> true
        let values = vec![create_number_value(-3.5)];
        let result = from_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test -0.0 -> false
        let values = vec![create_number_value(-0.0)];
        let result = from_number(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_from_number_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = from_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![create_number_value(5.0), create_number_value(3.0)];
        let result = from_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (bool)
        let values = vec![create_bool_value(true)];
        let result = from_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (string)
        let values = vec![create_string_value("hello")];
        let result = from_number(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = from_number(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_text_success() {
        let mut ctx = Context::new();

        // Test "true" -> true
        let values = vec![create_string_value("true")];
        let result = from_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test "false" -> false
        let values = vec![create_string_value("false")];
        let result = from_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }

        // Test "True" -> true (case insensitive)
        let values = vec![create_string_value("True")];
        let result = from_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test "FALSE" -> false (case insensitive)
        let values = vec![create_string_value("FALSE")];
        let result = from_text(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_from_text_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![create_string_value("true"), create_string_value("false")];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (bool)
        let values = vec![create_bool_value(true)];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (number)
        let values = vec![create_number_value(5.0)];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with unparseable text
        let values = vec![create_string_value("hello")];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with numeric string
        let values = vec![create_string_value("123")];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());

        // Test with empty string
        let values = vec![create_string_value("")];
        let result = from_text(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_equal_success() {
        let mut ctx = Context::new();

        // Test true == true -> true
        let values = vec![create_bool_value(true), create_bool_value(true)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test false == false -> true
        let values = vec![create_bool_value(false), create_bool_value(false)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test true == false -> false
        let values = vec![create_bool_value(true), create_bool_value(false)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }

        // Test false == true -> false
        let values = vec![create_bool_value(false), create_bool_value(true)];
        let result = is_equal(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_is_equal_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (one)
        let values = vec![create_bool_value(true)];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![
            create_bool_value(true),
            create_bool_value(false),
            create_bool_value(true),
        ];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (first parameter)
        let values = vec![create_number_value(5.0), create_bool_value(true)];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (second parameter)
        let values = vec![create_bool_value(true), create_string_value("hello")];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid values
        let values = vec![create_invalid_value(), create_bool_value(true)];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());

        let values = vec![create_bool_value(true), create_invalid_value()];
        let result = is_equal(&values, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_negate_success() {
        let mut ctx = Context::new();

        // Test !true -> false
        let values = vec![create_bool_value(true)];
        let result = negate(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }

        // Test !false -> true
        let values = vec![create_bool_value(false)];
        let result = negate(&values, &mut ctx).unwrap();
        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_negate_error() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (none)
        let values = vec![];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong number of parameters (too many)
        let values = vec![create_bool_value(true), create_bool_value(false)];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (number)
        let values = vec![create_number_value(5.0)];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());

        // Test with wrong value type (string)
        let values = vec![create_string_value("hello")];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = negate(&values, &mut ctx);
        assert!(result.is_err());
    }
}
