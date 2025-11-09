use tucana::shared::{Value, value::Kind};

use crate::context::signal::Signal;
use crate::{context::Context, error::RuntimeError, registry::HandlerFn};

pub fn collect_object_functions() -> Vec<(&'static str, HandlerFn)> {
    vec![
        ("std::object::contains_key", contains_key),
        ("std::object::keys", keys),
        ("std::object::size", size),
        ("std::object::set", set),
    ]
}

fn contains_key(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StructValue(object)),
        },
        Value {
            kind: Some(Kind::StringValue(key)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an object and a text as arguments but recieved: {:?}",
                values
            ),
        ));
    };

    let contains = object.fields.contains_key(key);

    Signal::Success(Value {
        kind: Some(Kind::BoolValue(contains)),
    })
}

fn size(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StructValue(object)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an object as an argument but received {:?}",
                values
            ),
        ));
    };
    Signal::Success(Value {
        kind: Some(Kind::NumberValue(object.fields.len() as f64)),
    })
}

fn keys(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StructValue(object)),
        },
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an object as an argument but received {:?}",
                values
            ),
        ));
    };

    let keys = object
        .fields
        .keys()
        .map(|key| Value {
            kind: Some(Kind::StringValue(key.clone())),
        })
        .collect::<Vec<Value>>();

    Signal::Success(Value {
        kind: Some(Kind::ListValue(tucana::shared::ListValue { values: keys })),
    })
}

fn set(values: &[Value], _ctx: &mut Context) -> Signal {
    let [
        Value {
            kind: Some(Kind::StructValue(object)),
        },
        Value {
            kind: Some(Kind::StringValue(key)),
        },
        value,
    ] = values
    else {
        return Signal::Failure(RuntimeError::simple(
            "InvalidArgumentRuntimeError",
            format!(
                "Expected an object as an argument but received {:?}",
                values
            ),
        ));
    };

    let mut new_object = object.clone();
    new_object.fields.insert(key.clone(), value.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(new_object)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use std::collections::HashMap;
    use tucana::shared::{Value, value::Kind};

    // Helper function to create a string value
    fn create_string_value(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }

    // Helper function to create a number value
    fn create_number_value(num: f64) -> Value {
        Value {
            kind: Some(Kind::NumberValue(num)),
        }
    }

    // Helper function to create a bool value
    fn create_bool_value(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }

    // Helper function to create an object/struct value
    fn create_object_value(fields: HashMap<String, Value>) -> Value {
        Value {
            kind: Some(Kind::StructValue(tucana::shared::Struct { fields })),
        }
    }

    // Helper function to create an empty object
    fn create_empty_object() -> Value {
        create_object_value(HashMap::new())
    }

    // Helper function to create a test object with some fields
    fn create_test_object() -> Value {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), create_string_value("John"));
        fields.insert("age".to_string(), create_number_value(30.0));
        fields.insert("active".to_string(), create_bool_value(true));
        create_object_value(fields)
    }

    // Helper function to create an invalid value (no kind)
    fn create_invalid_value() -> Value {
        Value { kind: None }
    }

    #[test]
    fn test_contains_key_success() {
        let mut ctx = Context::new();
        let test_object = create_test_object();

        // Test existing key
        let values = vec![test_object.clone(), create_string_value("name")];
        let signal = contains_key(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, true),
            _ => panic!("Expected BoolValue"),
        }

        // Test non-existing key
        let values = vec![test_object, create_string_value("nonexistent")];
        let signal = contains_key(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_contains_key_empty_object() {
        let mut ctx = Context::new();
        let empty_object = create_empty_object();

        let values = vec![empty_object, create_string_value("any_key")];
        let signal = contains_key(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::BoolValue(val)) => assert_eq!(val, false),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_contains_key_runtime_exception() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters
        let values = vec![create_test_object()];
        let result = contains_key(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong first parameter type (not an object)
        let values = vec![
            create_string_value("not_an_object"),
            create_string_value("key"),
        ];
        let result = contains_key(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong second parameter type (not a string)
        let values = vec![create_test_object(), create_number_value(123.0)];
        let result = contains_key(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with invalid values
        let values = vec![create_invalid_value(), create_string_value("key")];
        let result = contains_key(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with too many parameters
        let values = vec![
            create_test_object(),
            create_string_value("key"),
            create_string_value("extra"),
        ];
        let result = contains_key(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_size_success() {
        let mut ctx = Context::new();

        // Test with object containing fields
        let test_object = create_test_object();
        let values = vec![test_object];
        let signal = size(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 3.0), // name, age, active
            _ => panic!("Expected NumberValue"),
        }

        // Test with empty object
        let empty_object = create_empty_object();
        let values = vec![empty_object];
        let signal = size(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::NumberValue(val)) => assert_eq!(val, 0.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_size_runtime_exception() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (no parameters)
        let values = vec![];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong parameter type
        let values = vec![create_string_value("not_an_object")];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with too many parameters
        let values = vec![create_test_object(), create_string_value("extra")];
        let result = size(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_keys_success() {
        let mut ctx = Context::new();

        // Test with object containing fields
        let test_object = create_test_object();
        let values = vec![test_object];
        let signal = keys(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => {
                assert_eq!(list.values.len(), 3);

                // Convert to strings to check if all expected keys are present
                let mut key_strings: Vec<String> = list
                    .values
                    .iter()
                    .filter_map(|v| match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                key_strings.sort();

                let mut expected =
                    vec!["active".to_string(), "age".to_string(), "name".to_string()];
                expected.sort();

                assert_eq!(key_strings, expected);
            }
            _ => panic!("Expected ListValue"),
        }

        // Test with empty object
        let empty_object = create_empty_object();
        let values = vec![empty_object];
        let signal = keys(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::ListValue(list)) => assert_eq!(list.values.len(), 0),
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_keys_runtime_exception() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters
        let values = vec![];
        let result = keys(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong parameter type
        let values = vec![create_number_value(42.0)];
        let result = keys(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with invalid value
        let values = vec![create_invalid_value()];
        let result = keys(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with too many parameters
        let values = vec![create_test_object(), create_string_value("extra")];
        let result = keys(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_set_success() {
        let mut ctx = Context::new();

        // Test setting a new key
        let test_object = create_test_object();
        let values = vec![
            test_object.clone(),
            create_string_value("email"),
            create_string_value("john@example.com"),
        ];
        let signal = set(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::StructValue(struct_val)) => {
                assert_eq!(struct_val.fields.len(), 4); // original 3 + 1 new
                assert!(struct_val.fields.contains_key("email"));

                match struct_val.fields.get("email") {
                    Some(Value {
                        kind: Some(Kind::StringValue(email)),
                    }) => {
                        assert_eq!(email, "john@example.com");
                    }
                    _ => panic!("Expected email field to be a string"),
                }
            }
            _ => panic!("Expected StructValue"),
        }

        // Test overwriting an existing key
        let values = vec![
            test_object,
            create_string_value("age"),
            create_number_value(31.0),
        ];
        let signal = set(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::StructValue(struct_val)) => {
                assert_eq!(struct_val.fields.len(), 3); // same number of fields

                match struct_val.fields.get("age") {
                    Some(Value {
                        kind: Some(Kind::NumberValue(age)),
                    }) => {
                        assert_eq!(*age, 31.0);
                    }
                    _ => panic!("Expected age field to be a number"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_with_empty_object() {
        let mut ctx = Context::new();
        let empty_object = create_empty_object();

        let values = vec![
            empty_object,
            create_string_value("first_key"),
            create_bool_value(true),
        ];
        let signal = set(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::StructValue(struct_val)) => {
                assert_eq!(struct_val.fields.len(), 1);
                assert!(struct_val.fields.contains_key("first_key"));

                match struct_val.fields.get("first_key") {
                    Some(Value {
                        kind: Some(Kind::BoolValue(val)),
                    }) => {
                        assert_eq!(*val, true);
                    }
                    _ => panic!("Expected first_key field to be a boolean"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_with_different_value_types() {
        let mut ctx = Context::new();
        let test_object = create_test_object();

        // Test setting with a nested object
        let mut nested_fields = HashMap::new();
        nested_fields.insert("street".to_string(), create_string_value("123 Main St"));
        let nested_object = create_object_value(nested_fields);

        let values = vec![test_object, create_string_value("address"), nested_object];
        let signal = set(&values, &mut ctx);

        let result = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        match result.kind {
            Some(Kind::StructValue(struct_val)) => {
                assert!(struct_val.fields.contains_key("address"));

                match struct_val.fields.get("address") {
                    Some(Value {
                        kind: Some(Kind::StructValue(_)),
                    }) => {
                        // Successfully set nested object
                    }
                    _ => panic!("Expected address field to be a struct"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_runtime_exception() {
        let mut ctx = Context::new();

        // Test with wrong number of parameters (too few)
        let values = vec![create_test_object(), create_string_value("key")];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong first parameter type (not an object)
        let values = vec![
            create_string_value("not_an_object"),
            create_string_value("key"),
            create_string_value("value"),
        ];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with wrong second parameter type (not a string key)
        let values = vec![
            create_test_object(),
            create_number_value(123.0),
            create_string_value("value"),
        ];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with invalid values
        let values = vec![
            create_invalid_value(),
            create_string_value("key"),
            create_string_value("value"),
        ];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with no parameters
        let values = vec![];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));

        // Test with too many parameters
        let values = vec![
            create_test_object(),
            create_string_value("key"),
            create_string_value("value"),
            create_string_value("extra"),
        ];
        let result = set(&values, &mut ctx);
        assert_eq!(result, Signal::Failure(RuntimeError::default()));
    }

    #[test]
    fn test_set_preserves_original_object() {
        let mut ctx = Context::new();
        let original_object = create_test_object();

        // Get the original size
        let original_size = match &original_object.kind {
            Some(Kind::StructValue(struct_val)) => struct_val.fields.len(),
            _ => panic!("Expected StructValue"),
        };

        let values = vec![
            original_object.clone(),
            create_string_value("new_key"),
            create_string_value("new_value"),
        ];
        let signal = set(&values, &mut ctx);

        let _value = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success!"),
        };

        // Verify original object is unchanged
        match &original_object.kind {
            Some(Kind::StructValue(struct_val)) => {
                assert_eq!(struct_val.fields.len(), original_size);
                assert!(!struct_val.fields.contains_key("new_key"));
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_function_name_mapping() {
        // Test that the function names in collect_object_functions match expected patterns
        let functions = collect_object_functions();

        assert_eq!(functions.len(), 4);

        let function_names: Vec<&str> = functions.iter().map(|(name, _)| *name).collect();
        assert!(function_names.contains(&"std::object::contains_key"));
        assert!(function_names.contains(&"std::object::keys"));
        assert!(function_names.contains(&"std::object::size"));
        assert!(function_names.contains(&"std::object::set"));
    }
}
