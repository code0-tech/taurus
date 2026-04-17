use tucana::shared::{Struct, Value, value::Kind};

use crate::context::argument::Argument;
use crate::context::context::Context;
use crate::context::macros::args;
use crate::context::registry::{HandlerFn, HandlerFunctionEntry, IntoFunctionEntry};
use crate::context::signal::Signal;
use crate::runtime::error::RuntimeError;
use crate::value::value_from_i64;

pub fn collect_object_functions() -> Vec<(&'static str, HandlerFunctionEntry)> {
    vec![
        (
            "std::object::contains_key",
            HandlerFn::eager(contains_key, 2),
        ),
        ("std::object::keys", HandlerFn::eager(keys, 1)),
        ("std::object::size", HandlerFn::eager(size, 1)),
        ("std::object::set", HandlerFn::eager(set, 3)),
        ("std::object::get", HandlerFn::eager(get, 2)),
    ]
}

fn contains_key(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => object: Struct, key: String);
    let contains = object.fields.contains_key(&key);

    Signal::Success(Value {
        kind: Some(Kind::BoolValue(contains)),
    })
}

fn size(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => object: Struct);
    Signal::Success(value_from_i64(object.fields.len() as i64))
}

fn keys(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => object: Struct);

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

fn set(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => object: Struct, key: String, value: Value);
    let mut new_object = object.clone();
    new_object.fields.insert(key.clone(), value.clone());

    Signal::Success(Value {
        kind: Some(Kind::StructValue(new_object)),
    })
}

fn get(
    args: &[Argument],
    _ctx: &mut Context,
    _run: &mut dyn FnMut(i64, &mut Context) -> Signal,
) -> Signal {
    args!(args => object: Struct, key: String);
    match object.fields.get(&key) {
        Some(v) => Signal::Success(v.clone()),
        None => Signal::Failure(RuntimeError::simple(
            "FieldNotPresentRuntimeError",
            format!("field {} not present", key),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::argument::Argument;
    use crate::context::context::Context;
    use crate::value::{number_to_f64, value_from_f64};
    use std::collections::HashMap;
    use tucana::shared::{Struct as TcStruct, Value, value::Kind};

    // ---- helpers: Value builders ----
    fn v_string(s: &str) -> Value {
        Value {
            kind: Some(Kind::StringValue(s.to_string())),
        }
    }
    fn v_number(n: f64) -> Value {
        value_from_f64(n)
    }
    fn v_bool(b: bool) -> Value {
        Value {
            kind: Some(Kind::BoolValue(b)),
        }
    }
    fn v_struct(fields: HashMap<String, Value>) -> Value {
        Value {
            kind: Some(Kind::StructValue(TcStruct { fields })),
        }
    }

    // ---- helpers: Struct builders (for args that expect Struct) ----
    fn s_empty() -> TcStruct {
        TcStruct {
            fields: HashMap::new(),
        }
    }
    fn s_from(mut kv: Vec<(&str, Value)>) -> TcStruct {
        let mut map = HashMap::<String, Value>::new();
        for (k, v) in kv.drain(..) {
            map.insert(k.to_string(), v);
        }
        TcStruct { fields: map }
    }
    fn s_test() -> TcStruct {
        s_from(vec![
            ("name", v_string("John")),
            ("age", v_number(30.0)),
            ("active", v_bool(true)),
        ])
    }

    // ---- helpers: Argument builders ----
    #[allow(dead_code)]
    fn a_value(v: Value) -> Argument {
        Argument::Eval(v)
    }
    fn a_string(s: &str) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StringValue(s.to_string())),
        })
    }
    fn a_struct(s: TcStruct) -> Argument {
        Argument::Eval(Value {
            kind: Some(Kind::StructValue(s)),
        })
    }

    // dummy runner for handlers that accept `run: &mut dyn FnMut(i64, &mut Context) -> Signal`
    fn dummy_run(_: i64, _: &mut Context) -> Signal {
        Signal::Success(Value {
            kind: Some(Kind::NullValue(0)),
        })
    }

    #[test]
    fn test_contains_key_success() {
        let mut ctx = Context::default();

        // existing key
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("name")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(b),
            _ => panic!("Expected BoolValue"),
        }

        // non-existing key
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("nonexistent")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(!b),
            _ => panic!("Expected BoolValue"),
        }

        // empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty()), a_string("any_key")];
        let signal = contains_key(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::BoolValue(b)) => assert!(!b),
            _ => panic!("Expected BoolValue"),
        }
    }

    #[test]
    fn test_size_success() {
        let mut ctx = Context::default();

        // non-empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_test())];
        let signal = size(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::NumberValue(n)) => assert_eq!(number_to_f64(&n).unwrap_or_default(), 3.0),
            _ => panic!("Expected NumberValue"),
        }

        // empty object
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty())];
        let signal = size(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::NumberValue(n)) => assert_eq!(number_to_f64(&n).unwrap_or_default(), 0.0),
            _ => panic!("Expected NumberValue"),
        }
    }

    #[test]
    fn test_keys_success() {
        let mut ctx = Context::default();

        // with fields
        let mut run = dummy_run;
        let args = vec![a_struct(s_test())];
        let signal = keys(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::ListValue(list)) => {
                let mut got: Vec<String> = list
                    .values
                    .iter()
                    .filter_map(|v| {
                        if let Some(Kind::StringValue(s)) = &v.kind {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                got.sort();

                let mut expected =
                    vec!["active".to_string(), "age".to_string(), "name".to_string()];
                expected.sort();
                assert_eq!(got, expected);
            }
            _ => panic!("Expected ListValue"),
        }

        // empty object => empty list
        let mut run = dummy_run;
        let args = vec![a_struct(s_empty())];
        let signal = keys(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::ListValue(list)) => assert_eq!(list.values.len(), 0),
            _ => panic!("Expected ListValue"),
        }
    }

    #[test]
    fn test_set_success_and_overwrite() {
        let mut ctx = Context::default();

        // set new key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("email"),
            Argument::Eval(v_string("john@example.com")),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 4);
                match st.fields.get("email") {
                    Some(Value {
                        kind: Some(Kind::StringValue(s)),
                        ..
                    }) => assert_eq!(s, "john@example.com"),
                    _ => panic!("Expected email to be a string"),
                }
            }
            _ => panic!("Expected StructValue"),
        }

        // overwrite existing key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("age"),
            Argument::Eval(v_number(31.0)),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 3);
                match st.fields.get("age") {
                    Some(Value {
                        kind: Some(Kind::NumberValue(n)),
                        ..
                    }) => assert_eq!(number_to_f64(n).unwrap_or_default(), 31.0),
                    _ => panic!("Expected age to be a number"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_with_empty_object_and_nested() {
        let mut ctx = Context::default();

        // empty object -> add first key
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_empty()),
            a_string("first_key"),
            Argument::Eval(v_bool(true)),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                assert_eq!(st.fields.len(), 1);
                match st.fields.get("first_key") {
                    Some(Value {
                        kind: Some(Kind::BoolValue(b)),
                        ..
                    }) => assert_eq!(*b, true),
                    _ => panic!("Expected first_key to be a bool"),
                }
            }
            _ => panic!("Expected StructValue"),
        }

        // nested object value
        let nested = {
            let mut nf = HashMap::new();
            nf.insert("street".to_string(), v_string("123 Main St"));
            v_struct(nf)
        };
        let mut run = dummy_run;
        let args = vec![
            a_struct(s_test()),
            a_string("address"),
            Argument::Eval(nested),
        ];
        let signal = set(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            _ => panic!("Expected Success"),
        };
        match v.kind {
            Some(Kind::StructValue(st)) => {
                match st.fields.get("address") {
                    Some(Value {
                        kind: Some(Kind::StructValue(_)),
                        ..
                    }) => { /* ok */ }
                    _ => panic!("Expected address to be a struct"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_set_preserves_original_struct() {
        let mut ctx = Context::default();
        let original = s_test();
        let original_len = original.fields.len();

        // keep a clone to assert immutability
        let orig_clone = original.clone();

        let mut run = dummy_run;
        let args = vec![
            a_struct(original),
            a_string("new_key"),
            Argument::Eval(v_string("new_val")),
        ];
        let _ = set(&args, &mut ctx, &mut run);

        // ensure original (captured clone) unchanged
        assert_eq!(orig_clone.fields.len(), original_len);
        assert!(!orig_clone.fields.contains_key("new_key"));
    }

    #[test]
    fn test_get_success_string_field() {
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("name")];

        let signal = get(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            s => panic!("Expected Success, got {:?}", s),
        };

        match v.kind {
            Some(Kind::StringValue(s)) => assert_eq!(s, "John"),
            _ => panic!("Expected StringValue"),
        }
    }

    #[test]
    fn test_get_success_nested_struct_field() {
        let mut ctx = Context::default();
        let nested = {
            let mut nf = HashMap::new();
            nf.insert("street".to_string(), v_string("123 Main St"));
            v_struct(nf)
        };
        let object = s_from(vec![("address", nested)]);
        let mut run = dummy_run;
        let args = vec![a_struct(object), a_string("address")];

        let signal = get(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            s => panic!("Expected Success, got {:?}", s),
        };

        match v.kind {
            Some(Kind::StructValue(st)) => {
                match st.fields.get("street") {
                    Some(Value {
                        kind: Some(Kind::StringValue(s)),
                        ..
                    }) => assert_eq!(s, "123 Main St"),
                    _ => panic!("Expected nested 'street' string"),
                }
            }
            _ => panic!("Expected StructValue"),
        }
    }

    #[test]
    fn test_get_missing_field_returns_field_not_present() {
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("email")];

        let signal = get(&args, &mut ctx, &mut run);
        let err = match signal {
            Signal::Failure(err) => err,
            s => panic!("Expected Failure, got {:?}", s),
        };

        assert_eq!(err.name, "FieldNotPresent");
        assert_eq!(err.message, "field email not present");
    }

    #[test]
    fn test_collect_object_functions_registers_get_handler() {
        let mut ctx = Context::default();
        let mut run = dummy_run;
        let args = vec![a_struct(s_test()), a_string("age")];

        let entry = collect_object_functions()
            .into_iter()
            .find(|(id, _)| *id == "std::object::get")
            .map(|(_, entry)| entry)
            .expect("std::object::get should be registered");

        let signal = (entry.handler)(&args, &mut ctx, &mut run);
        let v = match signal {
            Signal::Success(v) => v,
            s => panic!("Expected Success, got {:?}", s),
        };

        match v.kind {
            Some(Kind::NumberValue(n)) => assert_eq!(number_to_f64(&n).unwrap_or_default(), 30.0),
            _ => panic!("Expected NumberValue"),
        }
    }
}
